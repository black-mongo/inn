//-------------------------------------------------------------------
// MIT License
// Copyright (c) 2022 black-mongo
// @author CameronYang
// @doc
//
// @end
// Created : 2022-04-14T11:19:56+08:00
//-------------------------------------------------------------------

use crate::codec::forward::ForwardCodec;
use crate::codec::{AuthChoice, BindStatus, DstAddress, VisitorRequest, VisitorResponse};
use crate::messages::*;
use crate::VisitorCodec;
use actix::prelude::*;
use actix::{Actor, Context};
use log::*;
use std::io;
use tokio::io::split;
use tokio::io::WriteHalf;
use tokio::net::TcpStream;
use tokio_util::codec::FramedRead;
type Framed<R, C> = actix::io::FramedWrite<R, WriteHalf<TcpStream>, C>;
#[derive(Default)]
pub struct VisitorSession {
    id: usize,
    server: Option<Recipient<ToProxyServer>>,
    forward: Option<Recipient<ToFoward>>,
    framed: Option<Framed<VisitorResponse, VisitorCodec>>,
}
impl VisitorSession {
    pub fn new(
        id: usize,
        server: Recipient<ToProxyServer>,
        framed: Framed<VisitorResponse, VisitorCodec>,
    ) -> Self {
        VisitorSession {
            id,
            forward: None,
            server: Some(server),
            framed: Some(framed),
        }
    }
    fn write(&mut self, resp: VisitorResponse) {
        if let Some(framed) = &mut self.framed {
            framed.write(resp);
        }
    }
    fn forward(&mut self, data: ToFoward) {
        if let Some(forward) = &mut self.forward {
            forward.do_send(data);
        }
    }
}
impl Actor for VisitorSession {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        if let Some(server) = &mut self.server {
            server.do_send(ToProxyServer::Connect(ctx.address().recipient()));
        }
        trace!("id = {}, Started", self.id);
    }
    fn stopped(&mut self, _ctx: &mut Self::Context) {
        trace!("id = {}, Stopped", self.id);
        self.forward(ToFoward::Stop);
    }
}
impl StreamHandler<Result<VisitorRequest, io::Error>> for VisitorSession {
    fn handle(&mut self, requst: Result<VisitorRequest, io::Error>, ctx: &mut Context<Self>) {
        // trace!("id = {}, stream handler: {:?}", self.id, requst);
        match requst {
            Ok(VisitorRequest::Greeting { proto, auth }) => {
                let _ = proto;
                let _ = auth;
                // if auth.len() == 1 && auth[0] == 0{
                self.write(VisitorResponse::Choice(AuthChoice::NoAuth));
                // }else{
                // self.write(VisitorResponse::Choice(AuthChoice::UserNamePwd));
                // }
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
                let addr = ctx.address().recipient();
                let ip = address.addr.clone();
                let port = address.port;
                let t = address.t;
                // Create remote connection
                // let forward = Forward::new();
                let _ = actix::spawn(async move {
                    if let Ok(stream) = TcpStream::connect(format!("{}:{}", ip.clone(), port)).await
                    {
                        Forward::create(|ctx1| {
                            let (r, w) = split(stream);
                            addr.do_send(ToSession::RemoteConnected(
                                ctx1.address().recipient(),
                                DstAddress::new(t, &ip, port),
                            ));
                            Forward::add_stream(FramedRead::new(r, ForwardCodec), ctx1);
                            Forward {
                                visitor: addr,
                                framed: Some(actix::io::FramedWrite::new(w, ForwardCodec, ctx1)),
                            }
                        });
                    } else {
                        addr.send(ToSession::RemoteConnectionRefuse).await.unwrap();
                    }
                });
            }
            Ok(VisitorRequest::Forward(data)) => {
                self.forward(ToFoward::Forward(data));
            }
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
        // trace!("id = {}, to_session = {:?} ", self.id, to);
        match to {
            ToSession::Ping => MessageResult(SessionReply::Pong),
            ToSession::Stop => MessageResult(SessionReply::Ok),
            ToSession::Meta => MessageResult(SessionReply::Meta(SessionMeta(self.id))),
            ToSession::Forward(data) => {
                self.write(VisitorResponse::Forward(data));
                MessageResult(SessionReply::Ok)
            }
            ToSession::RemoteConnected(recipient, dst) => {
                self.forward = Some(recipient);
                self.write(VisitorResponse::BindResp {
                    status: BindStatus::Granted,
                    address: Some(dst),
                });
                MessageResult(SessionReply::Ok)
            }
            ToSession::RemoteConnectionRefuse => {
                self.write(VisitorResponse::BindResp {
                    status: BindStatus::ConnectionRefuse,
                    address: None,
                });
                MessageResult(SessionReply::Ok)
            }
            ToSession::SetID(id) => {
                self.id = id;
                MessageResult(SessionReply::Ok)
            }
        }
    }
}

// Forward
pub struct Forward {
    visitor: Recipient<ToSession>,
    framed: Option<Framed<Vec<u8>, ForwardCodec>>,
}
impl Actor for Forward {
    type Context = Context<Self>;
    fn started(&mut self, _ctx: &mut Context<Self>) {
        trace!("forward started!");
    }
    fn stopped(&mut self, _ctx: &mut Context<Self>) {
        trace!("forward stopped");
        self.visitor.do_send(ToSession::Stop);
    }
}
impl Forward {
    #[allow(clippy::single_match)]
    pub fn write(&mut self, data: Vec<u8>) {
        match &mut self.framed {
            Some(framed) => framed.write(data),
            _ => {}
        }
    }
    pub fn visitor(&mut self, data: Vec<u8>) {
        self.visitor.do_send(ToSession::Forward(data));
    }
}
impl Handler<ToFoward> for Forward {
    type Result = MessageResult<ToFoward>;
    fn handle(&mut self, to: ToFoward, ctx: &mut Context<Self>) -> Self::Result {
        match to {
            ToFoward::Forward(data) => self.write(data),
            ToFoward::Stop => ctx.stop(),
        }
        MessageResult(ForwardReply::Ok)
    }
}
impl actix::io::WriteHandler<io::Error> for Forward {}
impl StreamHandler<Result<Vec<u8>, io::Error>> for Forward {
    fn handle(&mut self, resp: Result<Vec<u8>, io::Error>, _ctx: &mut Context<Self>) {
        // trace!("Forward handle receive = {:?}", resp);
        if let Ok(data) = resp {
            self.visitor(data);
        }
    }
}
