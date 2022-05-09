//-------------------------------------------------------------------
// MIT License
// Copyright (c) 2022 black-mongo
// @author CameronYang
// @doc
//
// @end
// Created : 2022-04-20T21:35:59+08:00
//-------------------------------------------------------------------

use actix::prelude::*;
use log::debug;
use network::{
    codec::{VisitorCodec, VisitorRequest, VisitorResponse},
    session::Framed,
};
use std::io::Error;
use tokio::sync::mpsc::Sender;
pub struct CliSession {
    pub sender: Sender<CliResponse>,
    pub framed: Framed<VisitorResponse, VisitorCodec>,
}
#[derive(Debug)]
pub enum CliResponse {
    Addr(Recipient<CliMessage>),
    Resp(common::cli::Cli),
}
impl Actor for CliSession {
    type Context = Context<Self>;
    fn started(&mut self, _ctx: &mut Self::Context) {
        debug!("Cli session started");
    }
    fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
        Running::Stop
    }
    fn stopped(&mut self, _ctx: &mut Self::Context) {
        debug!("Cli session stopped");
    }
}
impl actix::io::WriteHandler<Error> for CliSession {}
impl StreamHandler<Result<VisitorRequest, Error>> for CliSession {
    #[allow(clippy::single_match)]
    fn handle(&mut self, request: Result<VisitorRequest, Error>, _ctx: &mut Context<Self>) {
        debug!("cli reply = {:?}", request);
        let sender = self.sender.clone();
        match request {
            Ok(VisitorRequest::Cli(cli)) => {
                actix::spawn(async move {
                    let _ = sender.send(CliResponse::Resp(cli)).await;
                });
            }
            _ => {}
        }
    }
}
#[derive(Message, Debug)]
#[rtype(result = "()")]
pub enum CliMessage {
    Rpc(Vec<String>),
}
impl Handler<CliMessage> for CliSession {
    type Result = ();
    fn handle(&mut self, msg: CliMessage, _ctx: &mut Self::Context) -> Self::Result {
        debug!("handle={:?}", msg);
        let CliMessage::Rpc(rpc) = msg;
        let cli: common::cli::Cli = rpc.into();
        self.framed.write(VisitorResponse::Cli(cli));
    }
}
