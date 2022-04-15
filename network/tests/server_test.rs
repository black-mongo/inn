//-------------------------------------------------------------------
// MIT License
// Copyright (c) 2022 black-mongo
// @author CameronYang
// @doc
//
// @end
// Created : 2022-04-15T14:58:26+08:00
//-------------------------------------------------------------------
use actix::Actor;
use network::server::ProxyServer;
use network::*;
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
