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
use tokio::task::JoinHandle;
use tokio_util::codec::FramedRead;

pub struct NetWork;
use log::debug;
#[allow(clippy::unused_unit)]
impl NetWork {
    pub fn start<F>(&self, ip: &str, port: usize, listen_success: F) -> JoinHandle<()>
    where
        F: FnOnce() -> () + 'static,
    {
        let ip = format!("{}:{}", ip, port);
        let addr = SocketAddr::from_str(&ip).unwrap();
        let server = server::ProxyServer::default().start();
        actix::spawn(async move {
            let listener = TcpListener::bind(&addr).await.unwrap();
            listen_success();
            debug!("Listen {}", ip);
            while let Ok((stream, _)) = listener.accept().await {
                debug!("New Client comming");
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
        })
    }
}
