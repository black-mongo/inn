//-------------------------------------------------------------------
// MIT License
// Copyright (c) 2022 black-mongo
// @author CameronYang
// @doc
//
// @end
// Created : 2022-04-14T11:20:34+08:00
//-------------------------------------------------------------------

mod codec;
pub mod messages;
mod server;
pub mod session;
pub use messages::*;
pub use session::*;

use crate::codec::VisitorCodec;
use actix::prelude::StreamHandler;
use actix::Actor;
use actix_rt::net::TcpListener;
use std::net::SocketAddr;
use std::str::FromStr;
use tokio::io::split;
use tokio_util::codec::FramedRead;

pub struct NetWork;
impl NetWork {
    pub fn start(&self, ip: &str, port: usize) {
        let ip = format!("{}:{}", ip, port);
        let addr = SocketAddr::from_str(&ip).unwrap();
        let server = server::ProxyServer::default().start();
        // let server = &self.server;
        actix::spawn(async move {
            let listener = TcpListener::bind(&addr).await.unwrap();
            while let Ok((stream, _)) = listener.accept().await {
                VisitorSession::create(|ctx| {
                    let server = server.clone();
                    let (r, w) = split(stream);
                    VisitorSession::add_stream(FramedRead::new(r, VisitorCodec::default()), ctx);
                    VisitorSession::new(
                        0,
                        server.recipient(),
                        actix::io::FramedWrite::new(w, VisitorCodec::default(), ctx),
                    )
                });
            }
        });
    }
}
