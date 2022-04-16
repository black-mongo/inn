//-------------------------------------------------------------------
// MIT License
// Copyright (c) 2022 black-mongo
// @author CameronYang
// @doc
//
// @end
// Created : 2022-04-14T11:19:56+08:00
//-------------------------------------------------------------------

use crate::codec::{AuthChoice, BindStatus, VisitorRequest, VisitorResponse};
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
    fn write(&mut self, resp: VisitorResponse) {
        if let Some(framed) = &mut self.framed {
            framed.write(resp);
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
    fn handle(&mut self, requst: Result<VisitorRequest, io::Error>, ctx: &mut Context<Self>) {
        trace!("id = {}, stream handler: {:?}", self.id, requst);
        match requst {
            Ok(VisitorRequest::Greeting { proto, auth }) => {
                let _ = proto;
                let _ = auth;
                self.write(VisitorResponse::Choice(AuthChoice::UserNamePwd));
            }
            Ok(VisitorRequest::Auth { id, pwd }) => {
                if id == "admin" && pwd == "123456" {
                    self.write(VisitorResponse::AuthRespSuccess);
                } else {
                    self.write(VisitorResponse::AuthRespError);
                }
            }
            Ok(VisitorRequest::Connection { cmd, address }) => {
                let _ = cmd;
                // Create remote connection
                self.write(VisitorResponse::BindResp {
                    status: BindStatus::Granted,
                    address: Some(address),
                })
            }
            Ok(VisitorRequest::Forward(data)) => self.write(VisitorResponse::Forward(data)),
            e => {
                error!("stream handle error = {:?}, Stop session", e);
                ctx.stop();
            }
        }
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
