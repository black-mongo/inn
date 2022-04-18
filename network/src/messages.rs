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

use crate::codec::DstAddress;
// Command Message to session actor
#[derive(Debug)]
pub enum ToSession {
    Ping,
    Stop,
    Meta,
    SetID(usize),
    RemoteConnected(Recipient<ToFoward>, DstAddress),
    RemoteConnectionRefuse,
    Forward(Vec<u8>),
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
    Connect(Recipient<ToSession>),
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
#[derive(Debug, PartialEq)]
pub enum ToFoward {
    Stop,
    Forward(Vec<u8>),
}
pub enum ForwardReply {
    Ok,
}
impl Message for ToFoward {
    type Result = ForwardReply;
}
