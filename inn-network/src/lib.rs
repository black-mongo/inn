//-------------------------------------------------------------------
// MIT License
// Copyright (c) 2022 black-mongo
// @author CameronYang
// @doc
//
// @end
// Created : 2022-04-14T11:20:34+08:00
//-------------------------------------------------------------------
pub mod codec;
pub mod messages;
pub mod server;
pub mod session;
pub use messages::*;
use server::ProxyServer;
pub use session::*;
pub mod proxy;
use crate::codec::{VisitorCodec, T};
use actix::prelude::StreamHandler;
use actix::{Actor, Addr};
use actix_rt::net::TcpListener;
use std::net::SocketAddr;
use std::str::FromStr;
use tokio::io::split;
use tokio::task::JoinHandle;
use tokio_util::codec::FramedRead;

pub struct NetWork;
use log::info;
#[allow(clippy::unused_unit)]
impl NetWork {
    pub fn start<F>(
        &self,
        ip: &str,
        port: usize,
        listen_success: F,
        server: Addr<ProxyServer>,
    ) -> JoinHandle<()>
    where
        F: FnOnce() -> () + 'static,
    {
        let ip = format!("{}:{}", ip, port);
        let addr = SocketAddr::from_str(&ip).unwrap();
        actix::spawn(async move {
            let listener = TcpListener::bind(&addr).await.unwrap();
            listen_success();
            info!("Sock5 proxy server, Listening on socks5://{}", ip);
            while let Ok((stream, socket_addr)) = listener.accept().await {
                info!(
                    "New Client comming: ip={}, port={}",
                    socket_addr.ip(),
                    socket_addr.port()
                );
                VisitorSession::create(|ctx| {
                    let server = server.clone();
                    let (r, w) = split(stream);
                    VisitorSession::add_stream(FramedRead::new(r, VisitorCodec::default()), ctx);
                    VisitorSession::new(
                        0,
                        server.recipient(),
                        codec::DstAddress {
                            t: T::IPv4,
                            addr: socket_addr.ip().to_string(),
                            port: socket_addr.port(),
                        },
                        actix::io::FramedWrite::new(w, VisitorCodec::default(), ctx),
                    )
                });
            }
        })
    }
}
