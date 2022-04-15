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
// Ping message
pub struct Ping;

impl Message for Ping{
    type Result = Pong;
}
// Pong message
#[derive(Message, Default, Debug, PartialEq)]
#[rtype(result = "()")]
pub struct Pong;

// Get Session Meta
pub struct GetSessionMeta;

#[derive(Default, Debug, PartialEq)]
pub struct SessionMeta(pub u64);
impl Message for GetSessionMeta{
   type Result = SessionMeta; 
}
// Stop Session
#[derive(Message)]
#[rtype(result="()")]
pub struct StopSession;