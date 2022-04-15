//-------------------------------------------------------------------
// MIT License
// Copyright (c) 2022 black-mongo
// @author CameronYang
// @doc
//
// @end
// Created : 2022-04-15T11:16:30+08:00
//-------------------------------------------------------------------

use actix::Message;
use actix::Recipient;
// Command Message to session actor
#[derive(Debug)]
pub enum ToSession {
    Ping,
    Stop,
    Meta,
}
// Message reply from session actor
#[derive(Debug, PartialEq)]
pub enum SessionReply {
    Pong,
    Meta(SessionMeta),
    Ok,
}
impl Message for ToSession {
    type Result = SessionReply;
}

#[derive(Default, Debug, PartialEq)]
pub struct SessionMeta(pub usize);

#[derive(Debug)]
pub enum ToProxyServer {
    Connect(Recipient<ToProxyServer>),
    DisConnect(usize),
    OnlineCounter,
}
#[derive(Debug, PartialEq)]
pub enum ProxyServerReply {
    Id(usize),
    OnlineCounter(usize),
    Ok,
}
impl Message for ToProxyServer {
    type Result = ProxyServerReply;
}
