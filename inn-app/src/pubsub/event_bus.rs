use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use yew_agent::{Agent, AgentLink, Context, HandlerId};

pub struct EventBus {
    link: AgentLink<EventBus>,
    subscribers: HashSet<HandlerId>,
}
#[derive(Serialize, Deserialize, Debug)]
pub enum Publish {
    EventBusMsg(String),
}

impl Agent for EventBus {
    type Reach = Context<Self>;
    type Message = ();
    type Input = Publish;
    type Output = String;
    fn create(link: AgentLink<Self>) -> Self {
        Self {
            link,
            subscribers: HashSet::new(),
        }
    }
    fn update(&mut self, _msg: Self::Message) {}
    fn handle_input(&mut self, msg: Self::Input, _id: HandlerId) {
        match msg {
            Publish::EventBusMsg(msg) => {
                for sub in self.subscribers.iter() {
                    self.link.respond(*sub, msg.clone())
                }
            }
        }
    }
    fn connected(&mut self, id: HandlerId) {
        self.subscribers.insert(id);
    }

    fn disconnected(&mut self, id: HandlerId) {
        self.subscribers.remove(&id);
    }
}
