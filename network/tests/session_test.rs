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
async fn ping(){
    env_logger::init();
    let session = VisitorSession::new(1);
    let addr = session.start();
    let pong = addr.send(Ping).await.unwrap();
    assert_eq!(pong, Pong);
    addr.send(StopSession).await.unwrap();
}
#[actix_rt::test]
async fn get_session_meta(){
    let session = VisitorSession::new(1);
    let addr = session.start();
    let meta = addr.send(GetSessionMeta).await.unwrap();
    assert_eq!(meta, SessionMeta(1));
    addr.send(StopSession).await.unwrap();
}
#[actix_rt::test]
async fn stop_session(){
    let session = VisitorSession::new(1);
    let addr = session.start();
    let meta = addr.send(StopSession).await.unwrap();
    assert_eq!(meta, ());

}