//-------------------------------------------------------------------
// MIT License
// Copyright (c) 2022 black-mongo
// @author CameronYang
// @doc
//
// @end
// Created : 2022-04-14T11:19:56+08:00
//-------------------------------------------------------------------

use crate::codec::{VisitorRequest, VisitorResponse};
use crate::messages::*;
use crate::VisitorCodec;
use actix::prelude::*;
use actix::{Actor, Context};
use log::*;
use std::io;
use tokio::io::WriteHalf;
use tokio::net::TcpStream;
type Framed = actix::io::FramedWrite<VisitorResponse, WriteHalf<TcpStream>, VisitorCodec>;
#[derive(Default)]
pub struct VisitorSession {
    id: usize,
    server: Option<Recipient<ToProxyServer>>,
    framed: Option<Framed>,
}
impl VisitorSession {
    pub fn new(id: usize, server: Recipient<ToProxyServer>, framed: Framed) -> Self {
        VisitorSession {
            id,
            server: Some(server),
            framed: Some(framed),
        }
    }
}
impl Actor for VisitorSession {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        trace!("id = {}, Started", self.id);
    }
    fn stopped(&mut self, _ctx: &mut Self::Context) {
        trace!("id = {}, Stopped", self.id);
    }
}
impl StreamHandler<Result<VisitorRequest, io::Error>> for VisitorSession {
    fn handle(&mut self, _requst: Result<VisitorRequest, io::Error>, _ctx: &mut Context<Self>) {
        trace!("stream handler");
    }
    fn finished(&mut self, _ctx: &mut Self::Context) {
        trace!("finished");
    }
}
impl actix::io::WriteHandler<io::Error> for VisitorSession {}

impl Handler<ToSession> for VisitorSession {
    type Result = MessageResult<ToSession>;
    fn handle(&mut self, to: ToSession, _ctx: &mut Context<Self>) -> Self::Result {
        trace!("id = {}, to_session = {:?} ", self.id, to);
        match to {
            ToSession::Ping => MessageResult(SessionReply::Pong),
            ToSession::Stop => MessageResult(SessionReply::Ok),
            ToSession::Meta => MessageResult(SessionReply::Meta(SessionMeta(self.id))),
        }
    }
}
