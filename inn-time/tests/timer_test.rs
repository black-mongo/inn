//-------------------------------------------------------------------
// MIT License
// Copyright (c) 2022 black-mongo
// @author CameronYang
// @doc
//
// @end
// Created : 2022-04-14T11:23:45+08:00
//-------------------------------------------------------------------

use actix::AsyncContext;
use actix::Running;
use actix::{Actor, Context, Handler, System};
use inn_time::timer::RegisterTimer;
use inn_time::timer::TimerEvent;
use inn_time::timer::TimerType;
use inn_time::Timer;
use std::time::Duration;
#[derive(Default)]
pub struct TestActor(pub usize);

impl Actor for TestActor {
    type Context = Context<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {
        println!("Test Actor started");
        ctx.address().do_send(TimerEvent::TimeOut);
    }
    fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
        println!("test actor stopping");
        Running::Stop
    }
    fn stopped(&mut self, _ctx: &mut Self::Context) {
        println!("test actor stopped");
    }
}
impl Handler<TimerEvent> for TestActor {
    type Result = ();
    fn handle(&mut self, _event: TimerEvent, _ctx: &mut Context<Self>) -> Self::Result {
        self.0 += 1;
        if self.0 > 3 {
            System::current().stop();
        }
        println!("timerout ..");
    }
}

#[test]
fn regsiter() {
    let sys = System::new("event");
    let recipient = TestActor::default().start().recipient();
    let addr_timer = Timer::default().start();
    let register = RegisterTimer::new(
        "r1",
        Duration::from_secs(2),
        Timer::system_time() + Duration::from_secs(2),
        TimerType::Repeat,
        recipient,
    );
    let _res = addr_timer.do_send(register);
    sys.run().unwrap();
}
