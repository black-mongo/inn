//-------------------------------------------------------------------
// MIT License
// Copyright (c) 2022 black-mongo
// @author CameronYang
// @doc
//
// @end
// Created : 2022-04-15T11:23:35+08:00
//-------------------------------------------------------------------
use actix::prelude::*;
use network::*;
#[actix_rt::test]
async fn ping() {
    env_logger::init();
    let session = VisitorSession::default();
    let addr = session.start();
    let pong = addr.send(ToSession::Ping).await.unwrap();
    assert_eq!(pong, SessionReply::Pong);
    addr.send(ToSession::Stop).await.unwrap();
}
#[actix_rt::test]
async fn get_session_meta() {
    let session = VisitorSession::default();
    let addr = session.start();
    let meta = addr.send(ToSession::Meta).await.unwrap();
    assert_eq!(meta, SessionReply::Meta(SessionMeta(0)));
    addr.send(ToSession::Stop).await.unwrap();
}
#[actix_rt::test]
async fn stop_session() {
    let session = VisitorSession::default();
    let addr = session.start();
    let meta = addr.send(ToSession::Stop).await.unwrap();
    assert_eq!(meta, SessionReply::Ok);
}
