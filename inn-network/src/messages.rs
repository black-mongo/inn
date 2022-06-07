//-------------------------------------------------------------------
// MIT License
// Copyright (c) 2022 black-mongo
// @author CameronYang
// @doc
//
// @end
// Created : 2022-04-15T11:16:30+08:00
//-------------------------------------------------------------------

use std::collections::HashMap;

use actix::Message;
use actix::Recipient;
use serde::Deserialize;
use serde::Serialize;

use crate::codec::DstAddress;
// Command Message to session actor
#[derive(Debug, Clone)]
pub enum ToSession {
    Ping,
    Stop,
    Meta,
    RemoteConnected(Recipient<ToFoward>, DstAddress),
    RemoteConnectionRefuse,
    Forward(Vec<u8>),
    ProxyServerReply(ProxyServerReply),
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
    OnlineCounter(usize),
    Cli(usize, inn_common::cli::Cli),
    HttpReq(Box<WsHttpReq>),
}
#[derive(Debug, PartialEq, Clone)]
pub enum ProxyServerReply {
    Id(usize),
    OnlineCounter(usize),
    Ok,
    Cli(inn_common::cli::Cli),
    HttpReq(Box<WsHttpReq>),
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
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MsgName {
    HttpReq,
}
#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub struct WsHttpReq {
    pub id: String,
    pub uri: String,
    pub headers: HashMap<String, String>,
    pub status: u16,
    pub error: String,
    pub method: String,
    pub req_body: String,
    pub time: String,
    pub host: String,
    pub server_ip: String,
    pub protocol: String,
    pub resp_headers: HashMap<String, String>,
    pub resp_body: String,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct WsMsg<T> {
    pub msg_name: MsgName,
    pub msg: T,
}
