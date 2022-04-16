//-------------------------------------------------------------------
// MIT License
// Copyright (c) 2022 black-mongo
// @author CameronYang
// @doc
//
// @end
// Created : 2022-04-14T11:20:42+08:00
//-------------------------------------------------------------------
use actix::System;
use network::NetWork;
#[actix_rt::main]
async fn main() {
    NetWork.start("127.0.0.1", 4556, || {}).await;
    System::current().stop();
}
