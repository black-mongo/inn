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
impl Handler<Ping> for VisitorSession {
    type Result = MessageResult<Ping>;
    fn handle(&mut self, _ping: Ping, _ctx: &mut Context<Self>) -> Self::Result{
        trace!("id = {}, Ping", self.id);
        MessageResult(Pong)
    }
}
impl Handler<StopSession> for VisitorSession{
    type Result = ();
    fn handle(&mut self, _: StopSession, ctx: &mut Context<Self>) -> Self::Result{
        trace!("id = {}, StopSession", self.id);
        ctx.stop(); 
    }
}
impl Handler<GetSessionMeta> for VisitorSession{
    type Result = MessageResult<GetSessionMeta>;
    fn handle(&mut self, _: GetSessionMeta, _ctx: &mut Context<Self>) -> Self::Result{
        MessageResult(SessionMeta(self.id))
    }
}
