//-------------------------------------------------------------------
// MIT License
// Copyright (c) 2022 black-mongo
// @author CameronYang
// @doc
//
// @end
// Created : 2022-04-14T11:20:42+08:00
//-------------------------------------------------------------------
use actix::Actor;
use inn_network::server::ProxyServer;
use inn_network::ws;
use inn_network::{proxy::Proxy, NetWork};
#[actix_rt::main]
async fn main() {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));
    //
    let server = ProxyServer::default().start();
    let s = server.clone();
    let _ = NetWork.start("127.0.0.1", 4556, || {}, server.clone());
    actix::spawn(async move {
        Proxy::start_proxy(
            "127.0.0.1:4557",
            "ca/ca/cacert.pem",
            "ca/ca/cakey.pem",
            s.clone(),
        )
        .await
    });
    let _ = ws::run("127.0.0.1".to_string(), 4558, server.clone()).await;
}
