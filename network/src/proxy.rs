//-------------------------------------------------------------------
// MIT License
// Copyright (c) 2022 black-mongo
// @author CameronYang
// @doc
//
// @end
// Created : 2022-05-25T00:49:22+08:00
//-------------------------------------------------------------------

use std::convert::Infallible;
use std::sync::Arc;

use common::genca::CertAuthority;
use http::uri::PathAndQuery;
use hyper::client::HttpConnector;
use hyper::server::conn::{AddrStream, Http};
use hyper::service::{make_service_fn, service_fn};
use hyper::upgrade::Upgraded;
use hyper::{Body, Client, Method, Request, Response, Server};
use hyper_rustls::{HttpsConnector, HttpsConnectorBuilder};
use log::{debug, error, info};
use tokio::net::TcpStream;
use tokio_rustls::TlsAcceptor;
#[derive(Clone)]
pub struct Proxy {
    ca: Arc<CertAuthority>,
    client: Client<HttpsConnector<HttpConnector>>,
}
// To try this example:
// 1. cargo run --example http_proxy
// 2. config http_proxy in command line
//    $ export http_proxy=http://127.0.0.1:8100
//    $ export https_proxy=http://127.0.0.1:8100
// 3. send requests
//    $ curl -i https://www.some_domain.com/
impl Proxy {
    pub async fn start_proxy(ip: &str, cacert: &str, cakey: &str) {
        let addr = ip.parse().expect("invalid ip");
        let https = HttpsConnectorBuilder::new()
            .with_webpki_roots()
            .https_or_http()
            .enable_http1()
            .enable_http2()
            .build();
        let client = Client::builder()
            .http1_title_case_headers(true)
            .http1_preserve_header_case(true)
            .build(https);
        let ca = Arc::new(CertAuthority::new(cacert.to_string(), cakey.to_string()));
        let make_service = make_service_fn(move |_conn: &AddrStream| {
            let client = client.clone();
            let ca = Arc::clone(&ca);
            async move {
                Ok::<_, Infallible>(service_fn(move |req| {
                    Proxy {
                        ca: Arc::clone(&ca),
                        client: client.clone(),
                    }
                    .proxy(req)
                }))
            }
        });

        let server = Server::bind(&addr)
            .http1_preserve_header_case(true)
            .http1_title_case_headers(true)
            .serve(make_service);

        info!("Https proxy server, Listening on http://{}", addr);

        if let Err(e) = server.await {
            eprintln!("server error: {}", e);
        }
    }

    async fn proxy(self, req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
        if Method::CONNECT == req.method() {
            // Received an HTTP request like:
            // ```
            // CONNECT www.domain.com:443 HTTP/1.1
            // Host: www.domain.com:443
            // Proxy-Connection: Keep-Alive
            // ```
            //
            // When HTTP method is CONNECT we should return an empty body
            // then we can eventually upgrade the connection and talk a new protocol.
            //
            // Note: only after client received an empty body with STATUS_OK can the
            // connection be upgraded, so we can't return a response inside
            // `on_upgrade` future.
            if let Some(addr) = Proxy::host_addr(req.uri()) {
                tokio::task::spawn(async move {
                    match hyper::upgrade::on(req).await {
                        Ok(upgraded) => {
                            let host: Vec<&str> = addr.split(':').collect();
                            if Proxy::mitm_match(host[0], host[1]) {
                                // Man in the middle
                                let server_config = self.ca.dynamic_gen_cert(host[0]).await;
                                match TlsAcceptor::from(server_config).accept(upgraded).await {
                                    Ok(stream) => {
                                        if let Err(e) = self.serve_https(stream).await {
                                            error!("addr = {} serve_https error = {}", addr, e);
                                        }
                                    }
                                    Err(e) => {
                                        error!("addr = {} TlsAcceptor error = {}", addr, e);
                                    }
                                }
                            } else {
                                debug!("addr = {}, tunnel", addr);
                                let _ = Proxy::tunnel(upgraded, &addr).await;
                            }
                        }
                        Err(e) => error!("addr = {}, upgrade error: {}", addr, e),
                    }
                });
                Ok(Response::new(Body::empty()))
            } else {
                error!("CONNECT host is not socket addr: {:?}", req.uri());
                let mut resp = Response::new(Body::from("CONNECT must be to a socket address"));
                *resp.status_mut() = http::StatusCode::BAD_REQUEST;
                Ok(resp)
            }
        } else {
            self.client.request(req).await
        }
    }
    async fn serve_https(
        self,
        stream: tokio_rustls::server::TlsStream<Upgraded>,
    ) -> Result<(), hyper::Error> {
        let service = service_fn(|mut req| {
            if req.version() == http::Version::HTTP_10 || req.version() == http::Version::HTTP_11 {
                let authority = req
                    .headers()
                    .get(http::header::HOST)
                    .expect("Host is a required header")
                    .to_str()
                    .expect("Failed to convert host to str");

                let uri = http::uri::Builder::new()
                    .scheme(http::uri::Scheme::HTTPS)
                    .authority(authority)
                    .path_and_query(
                        req.uri()
                            .path_and_query()
                            .unwrap_or(&PathAndQuery::from_static("/"))
                            .to_owned(),
                    )
                    .build()
                    .expect("Failed to build URI");

                let (mut parts, body) = req.into_parts();
                parts.uri = uri;
                req = Request::from_parts(parts, body)
            };
            debug!("uri = {}, headers = {:?}", req.uri(), req.headers());
            self.client.request(req)
        });

        Http::new()
            .serve_connection(stream, service)
            .with_upgrades()
            .await
    }
    fn host_addr(uri: &http::Uri) -> Option<String> {
        uri.authority().map(|auth| auth.to_string())
    }

    // Create a TCP connection to host:port, build a tunnel between the connection and
    // the upgraded connection
    async fn tunnel(mut upgraded: Upgraded, addr: &str) -> std::io::Result<()> {
        // Connect to remote server
        let mut server = TcpStream::connect(addr).await?;
        let _ = tokio::io::copy_bidirectional(&mut upgraded, &mut server).await?;
        Ok(())
    }
    fn mitm_match(host: &str, port: &str) -> bool {
        matches!((host, port), ("github.com", _) | ("www.github.com", _))
    }
}
