//-------------------------------------------------------------------
// MIT License
// Copyright (c) 2022 black-mongo
// @author CameronYang
// @doc
//
// @end
// Created : 2022-06-07T10:39:36+08:00
//-------------------------------------------------------------------
use actix::{Actor, Addr, AsyncContext, Handler, MessageResult, StreamHandler};
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use log::{error, info};

use crate::{
    server::ProxyServer, MsgName, ProxyServerReply, SessionReply, ToProxyServer, ToSession, WsMsg,
};

/// Define HTTP actor
pub struct Ws {
    id: usize,
    server: Addr<ProxyServer>,
}

impl Actor for Ws {
    type Context = ws::WebsocketContext<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {
        self.server
            .do_send(ToProxyServer::Connect(ctx.address().recipient()));
    }
    fn stopped(&mut self, _ctx: &mut Self::Context) {
        self.server.do_send(ToProxyServer::DisConnect(self.id));
    }
}
impl Handler<ToSession> for Ws {
    type Result = MessageResult<ToSession>;
    fn handle(&mut self, to: ToSession, ctx: &mut Self::Context) -> Self::Result {
        match to {
            ToSession::ProxyServerReply(ProxyServerReply::Id(id)) => {
                self.id = id;
                MessageResult(SessionReply::Ok)
            }
            ToSession::ProxyServerReply(ProxyServerReply::OnlineCounter(_n)) => {
                MessageResult(SessionReply::Ok)
            }
            ToSession::ProxyServerReply(ProxyServerReply::HttpReq(req)) => {
                let ws_msg = WsMsg {
                    msg_name: MsgName::HttpReq,
                    msg: req,
                };
                let payload = serde_json::to_string(&ws_msg).unwrap();
                ctx.text(payload);
                MessageResult(SessionReply::Ok)
            }
            _ => MessageResult(SessionReply::Ok),
        }
    }
}

/// Handler for ws::Message message
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for Ws {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => ctx.text(text),
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            _ => (),
        }
    }
}

async fn index(
    req: HttpRequest,
    stream: web::Payload,
    server: web::Data<Addr<ProxyServer>>,
) -> Result<HttpResponse, Error> {
    let resp = ws::start(
        Ws {
            id: 0,
            server: server.get_ref().clone(),
        },
        &req,
        stream,
    );
    println!("{:?}", resp);
    resp
}

pub async fn run(ip: String, port: u16, server: Addr<ProxyServer>) -> std::io::Result<()> {
    info!("Listening on ws://{}:{}/ws/", ip.clone(), port);
    let rs = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(server.clone()))
            .route("/ws/", web::get().to(index))
    })
    .bind((ip.clone(), port))?
    .run()
    .await;
    match &rs {
        Ok(()) => {}
        Err(e) => {
            error!("start ws://{}:{} error:{}", ip, port, e)
        }
    }
    rs
}
