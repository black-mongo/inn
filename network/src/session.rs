//-------------------------------------------------------------------
// MIT License
// Copyright (c) 2022 black-mongo
// @author CameronYang
// @doc
//
// @end
// Created : 2022-04-14T11:19:56+08:00
//-------------------------------------------------------------------

use actix::prelude::*;
use actix::{Actor, Context};
use crate::messages::*;
use log::*;
pub struct VisitorSession {
    id: u64,
}
impl VisitorSession {
    pub fn new(id: u64) -> Self {
        VisitorSession {
            id,
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
impl Handler<ToSession> for VisitorSession {
    type Result = MessageResult<ToSession>;
    fn handle(&mut self, to: ToSession, _ctx: &mut Context<Self>) -> Self::Result{
        trace!("id = {}, to_session = {:?} ", self.id, to);
        match to{
            ToSession::Ping => MessageResult(SessionReply::Pong),
            ToSession::Stop => MessageResult(SessionReply::Ok),
            ToSession::Meta => MessageResult(SessionReply::Meta(SessionMeta(self.id))),
        }
    }
}