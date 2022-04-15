//-------------------------------------------------------------------
// MIT License
// Copyright (c) 2022 black-mongo
// @author CameronYang
// @doc
//
// @end
// Created : 2022-04-14T11:20:27+08:00
//-------------------------------------------------------------------

use actix::prelude::*;
use actix::{Actor, Context, Message};
use core::ops::Bound::Included;
use std::cmp::Ord;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::time::Duration;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

// Timer Actor
#[derive(Default)]
pub struct Timer {
    key: HashMap<String, Duration>,
    registers: BTreeMap<Duration, Vec<RegisterTimer>>,
}
// Register timer
#[derive(Message)]
#[rtype(result = "()")]
#[derive(Eq, Clone)]
pub struct RegisterTimer {
    name: String,
    interval: Duration,
    expired: Duration, //
    t: TimerType,
    recipient: Recipient<TimerEvent>,
}
// Timeout Event
#[derive(Message)]
#[rtype(result = "()")]
pub enum TimerEvent {
    TimeOut,
}
#[derive(Message)]
#[rtype(result = "()")]
#[derive(Eq, Clone)]
pub enum TimerType {
    Once,
    Repeat,
}
// tick
#[derive(Message)]
#[rtype(result = "()")]
struct Tick;
impl Timer {
    pub fn run(&self, ctx: &mut Context<Self>, register: &RegisterTimer) {
        register.run(ctx);
        match register.t {
            TimerType::Once => {}
            TimerType::Repeat => {
                let mut register: RegisterTimer = register.clone();
                register.next();
                let addr = ctx.address();
                // send register again to self
                addr.do_send(register);
            }
        }
    }
    pub fn system_time() -> Duration {
        let start = SystemTime::now();
        start
            .duration_since(UNIX_EPOCH)
            .expect("Clock may have gone backwards")
    }
    fn tick(&mut self, ctx: &mut Context<Self>) {
        ctx.run_later(Duration::from_nanos(1), |_act, ctx| {
            let addr = ctx.address();
            addr.do_send(Tick);
        });
    }
}
impl Actor for Timer {
    type Context = Context<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {
        self.tick(ctx);
    }
}

// RegiterTimer Message
impl Handler<RegisterTimer> for Timer {
    type Result = ();
    fn handle(&mut self, register_timer: RegisterTimer, _ctx: &mut Context<Self>) -> Self::Result {
        match (
            self.key.get_mut(&register_timer.name),
            self.registers.get_mut(&register_timer.expired),
        ) {
            (None, None) => {
                self.key
                    .insert(register_timer.name.clone(), register_timer.expired);
                self.registers
                    .insert(register_timer.expired, vec![register_timer]);
            }
            (Some(expired), None) => {
                *expired = register_timer.expired;
                self.registers
                    .insert(register_timer.expired, vec![register_timer]);
            }
            (None, Some(list)) => {
                self.key
                    .insert(register_timer.name.clone(), register_timer.expired);
                let mut is_exist = false;
                for old in list.iter_mut() {
                    if old.name == register_timer.name {
                        *old = register_timer.clone();
                        is_exist = true;
                        break;
                    }
                }
                if !is_exist {
                    list.push(register_timer);
                }
            }
            (Some(expired), Some(list)) => {
                *expired = register_timer.expired;
                for old in list.iter_mut() {
                    if old.name == register_timer.name {
                        *old = register_timer;
                        break;
                    }
                }
            }
        }
    }
}

impl Handler<Tick> for Timer {
    type Result = ();
    fn handle(&mut self, _msg: Tick, ctx: &mut Context<Self>) {
        // check timeout timer
        let mut temp = vec![];
        let now = Timer::system_time();
        for (key, list) in self
            .registers
            .range((Included(&Duration::from_millis(0)), Included(&now)))
        {
            // do task
            for register in list {
                self.key.remove(&register.name.clone());
                self.run(ctx, register);
            }
            temp.push(*key);
        }
        for key in temp {
            self.registers.remove(&key);
        }
        // send tick to self
        self.tick(ctx);
    }
}

impl PartialEq for TimerType {
    fn eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl RegisterTimer {
    pub fn new(
        name: &str,
        interval: Duration,
        expired: Duration,
        t: TimerType,
        recipient: Recipient<TimerEvent>,
    ) -> Self {
        RegisterTimer {
            name: name.to_string(),
            interval,
            expired,
            t,
            recipient,
        }
    }
    pub fn run(&self, _ctx: &mut Context<Timer>) {
        let who = self.recipient.clone();
        Arbiter::spawn(async move {
            let _rs = who.do_send(TimerEvent::TimeOut);
        });
    }
    pub fn next(&mut self) {
        self.expired += self.interval;
    }
}
impl Ord for RegisterTimer {
    fn cmp(&self, other: &Self) -> Ordering {
        self.expired.cmp(&other.expired)
    }
}
impl PartialOrd for RegisterTimer {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl PartialEq for RegisterTimer {
    fn eq(&self, other: &Self) -> bool {
        self.expired == other.expired
    }
}
