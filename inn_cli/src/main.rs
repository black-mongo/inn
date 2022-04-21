//-------------------------------------------------------------------
// MIT License
// Copyright (c) 2022 black-mongo
// @author CameronYang
// @doc
//
// @end
// Created : 2022-04-18T23:21:58+08:00
//-------------------------------------------------------------------
mod session;
use actix::clock::sleep;
use actix::{Actor, Recipient, StreamHandler};
use log::info;
use network::codec::VisitorCodec;
use session::{CliMessage, CliResponse};
use tokio::io::split;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio_util::codec::FramedRead;

use crate::session::CliSession;
#[actix_rt::main]
async fn main() {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));
    let ip = "127.0.0.1";
    let port = 4556;
    let (tx, mut rx) = mpsc::channel::<CliResponse>(10);
    let _ = actix::spawn(async move {
        if let Ok(stream) = TcpStream::connect(format!("{}:{}", ip, port)).await {
            let addr = CliSession::create(|ctx1| {
                let (r, w) = split(stream);
                CliSession::add_stream(FramedRead::new(r, VisitorCodec::default()), ctx1);
                CliSession {
                    sender: tx.clone(),
                    framed: actix::io::FramedWrite::new(w, VisitorCodec::default(), ctx1),
                }
            });
            let _ = tx.send(CliResponse::Addr(addr.recipient())).await;
        }
    })
    .await;
    let mut session: Option<Recipient<CliMessage>> = None;
    let mut n = 1;
    // receive
    loop {
        let rs = rx.try_recv();
        match rs {
            Ok(CliResponse::Addr(addr)) => {
                session = Some(addr);
            }
            Ok(CliResponse::Resp(cli)) => {
                info!("main {:?}", cli);

                if let Some(addr) = &session {
                    addr.do_send(CliMessage::Rpc(vec!["stop".into()]));
                    sleep(std::time::Duration::from_millis(200)).await;
                }
                break;
            }
            Err(tokio::sync::mpsc::error::TryRecvError::Empty) => {
                sleep(std::time::Duration::from_nanos(1)).await;
            }
            _ => {}
        }
        n += 1;
        if let Some(addr) = &session {
            if n == 2 {
                addr.do_send(CliMessage::Rpc(vec!["online_counter".into()]));
            }
        }
    }
}
