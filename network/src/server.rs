//-------------------------------------------------------------------
// MIT License
// Copyright (c) 2022 black-mongo
// @author CameronYang
// @doc
//
// @end
// Created : 2022-04-15T12:32:45+08:00
//-------------------------------------------------------------------

use crate::messages::*;
use actix::prelude::*;
use actix::{Actor, Context, Handler};
use rand::rngs::ThreadRng;
use rand::Rng;
use std::collections::HashMap;
pub struct ProxyServer {
    sessions: HashMap<usize, Recipient<ToProxyServer>>,
    rng: ThreadRng,
}
impl Default for ProxyServer {
    fn default() -> Self {
        ProxyServer {
            sessions: HashMap::new(),
            rng: rand::thread_rng(),
        }
    }
}
impl Actor for ProxyServer {
    type Context = Context<Self>;
    fn started(&mut self, _ctx: &mut Self::Context) {}
}
impl Handler<ToProxyServer> for ProxyServer {
    type Result = MessageResult<ToProxyServer>;
    fn handle(&mut self, command: ToProxyServer, _ctx: &mut Context<Self>) -> Self::Result {
        match command {
            ToProxyServer::Connect(recipient) => {
                let id = self.rng.gen::<usize>();
                self.sessions.insert(id, recipient);
                MessageResult(ProxyServerReply::Id(id))
            }
            ToProxyServer::DisConnect(id) => {
                self.sessions.remove(&id);
                MessageResult(ProxyServerReply::Ok)
            }
            ToProxyServer::OnlineCounter => {
                MessageResult(ProxyServerReply::OnlineCounter(self.sessions.len()))
            }
        }
    }
}
#[cfg(test)]
mod test {
    use super::*;
    #[actix_rt::test]
    async fn new_session() {
        let addr = ProxyServer::default().start();
        let counter = addr.send(ToProxyServer::OnlineCounter).await.unwrap();
        assert_eq!(counter, ProxyServerReply::OnlineCounter(0));
        let id = addr
            .send(ToProxyServer::Connect(addr.clone().recipient()))
            .await
            .unwrap();
        match id {
            ProxyServerReply::Id(id) => {
                let counter = addr.send(ToProxyServer::OnlineCounter).await.unwrap();
                assert_eq!(counter, ProxyServerReply::OnlineCounter(1));
                let rs = addr.send(ToProxyServer::DisConnect(id)).await.unwrap();
                assert_eq!(rs, ProxyServerReply::Ok);
                let counter = addr.send(ToProxyServer::OnlineCounter).await.unwrap();
                assert_eq!(counter, ProxyServerReply::OnlineCounter(0));
            }
            e => {
                panic!("{:?}", e);
            }
        }
    }
}
