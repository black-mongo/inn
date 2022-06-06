//-------------------------------------------------------------------
// MIT License
// Copyright (c) 2022 black-mongo
// @author CameronYang
// @doc
//
// @end
// Created : 2022-04-14T11:20:42+08:00
//-------------------------------------------------------------------
use actix::{Actor, System};
use inn_network::server::ProxyServer;
use inn_network::{proxy::Proxy, NetWork};
#[actix_rt::main]
async fn main() {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));
    //
    let server = ProxyServer::default().start();
    let sock5 = async {
        let _ = NetWork
            .start("127.0.0.1", 4556, || {}, server.clone())
            .await;
    };
    let http_proxy = async {
        Proxy::start_proxy(
            "127.0.0.1:4557",
            "ca/ca/cacert.pem",
            "ca/ca/cakey.pem",
            server.clone(),
        )
        .await;
    };
    tokio::join!(sock5, http_proxy);
    System::current().stop();
}
