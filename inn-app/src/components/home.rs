use std::{collections::HashMap, vec};

use serde_json::Value;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};

use super::app::{AppContext, Route};
use crate::{network::WebsocketService, pubsub::EventBus};
use serde::{Deserialize, Serialize};
use yew_router::prelude::*;

#[derive(Clone)]
struct UserProfile {
    name: String,
    avatar: String,
}

pub struct Home {
    ws: WebsocketService,
    users: Vec<UserProfile>,
    input: NodeRef,
    messages: Vec<WsHttpReq>,
    _producer: Box<dyn Bridge<EventBus>>,
}
#[derive(Debug)]
pub enum Msg {
    HandleMsg(String),
}
#[derive(Deserialize)]
struct MessageData {
    from: String,
    message: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MsgTypes {
    Users,
    Register,
    Message,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MsgName {
    HttpReq,
}
#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub struct WsHttpReq {
    pub id: String,
    pub uri: String,
    pub headers: HashMap<String, String>,
    pub status: u16,
    pub error: String,
    pub method: String,
    pub req_body: String,
    pub time: String,
    pub host: String,
    pub server_ip: String,
    pub protocol: String,
    pub resp_headers: HashMap<String, String>,
    pub resp_body: String,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct WsMsg<T> {
    pub msg_name: MsgName,
    pub msg: T,
}
impl Component for Home {
    type Message = Msg;
    type Properties = ();
    fn create(ctx: &Context<Self>) -> Self {
        log::debug!("home create");
        let (app_ctx, _) = ctx
            .link()
            .context::<AppContext>(Callback::noop())
            .expect("context to be set");
        // let message = WebSocketMessage {
        //     message_type: MsgTypes::Register,
        //     data: Some(app_ctx.user.clone().into_inner()),
        //     data_array: None,
        // };
        let ws = WebsocketService::new();
        // if let Ok(_) = ws
        //     .tx
        //     .clone()
        //     .try_send(serde_json::to_string(&message).unwrap())
        // {
        //     log::debug!("message sent successfully");
        // }
        Self {
            ws,
            users: vec![],
            messages: vec![],
            input: NodeRef::default(),
            _producer: EventBus::bridge(ctx.link().callback(Msg::HandleMsg)),
        }
    }
    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        log::debug!("{:?}", msg);
        match msg {
            Msg::HandleMsg(s) => {
                let msg: Value = serde_json::from_str(&s).unwrap();
                match &msg["msg_name"] {
                    Value::String(s) => {
                        let msg_name: MsgName =
                            serde_json::from_value(Value::String(s.clone())).unwrap();
                        match msg_name {
                            MsgName::HttpReq => {
                                let msg: WsHttpReq =
                                    serde_json::from_value(msg["msg"].clone()).unwrap();
                                self.messages.push(msg);
                                return true;
                            }
                        }
                    }
                    _ => {
                        return false;
                    }
                }
            }
        }
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        // let submit = ctx.link().callback(|_| Msg::SubmitMessage);
        for row in self.messages.iter() {
            log::debug!("{:?}", row)
        }
        let (_, list) = self.messages.iter().fold((0_i32,Vec::<Html>::new()),|(i,mut acc),m|{
                    let a = html!{
                        <table class="table-fixed w-full border border-t-0 border-gray-200 text-left">
                        <tbody>
                <tr class="broder hover:bg-black hover:text-white hover:cursor-pointer overflow-hidden whitespace-nowrap overflow-ellipsis">
                    <td class="w-16">
                        {i}
                    </td>
                    <td class="w-16">
                        {m.status}
                    </td>
                    <td class="w-20">
                        {m.method.clone()}
                    </td>
                    <td class="w-24">
                        {m.protocol.clone()}
                    </td>

                    <td class="w-28">
                        {m.server_ip.clone()}
                    </td>
                    <td class="w-36">
                        {m.host.clone()}
                    </td>
                    <td class="">
                        <div class="overflow-hidden whitespace-nowrap overflow-ellipsis">
                        {m.uri.clone()}
                        </div>
                    </td>

                    <td class="w-32">
                        {m.status}
                    </td>
                    <td class="w-20">
                        {m.time.clone()}
                    </td>
                </tr>
                </tbody>
                </table>
            };
            acc.push(a);
            (i+1, acc)
            });
        html! {
           <>
           <div class="overflow-auto">
           <table class="table-fixed w-full border border-b-1 border-gray-200 text-left text-gray-600">
           <thead >
               <tr>
                   <th class="w-16">
                      {"#"}
                   </th>
                   <th class="w-16">
                      {"Result"}
                   </th>
                   <th class="w-20">
                       {"Method"}
                   </th>
                   <th class="w-24">
                       {"Protocol"}
                   </th>
                   <th class="w-28">
                       {"ServerIP"}
                   </th>
                   <th class="w-36">
                       {"Host"}
                   </th>
                   <th class="">
                       {"URL"}
                   </th>
                   <th class="w-32">
                       {"Type"}
                   </th>
                   <th class="w-20">
                       {"Time"}
                   </th>
               </tr>
           </thead>
           </table>
           {
               list.into_iter().rev().collect::<Html>()
           }
        </div>
        </>
           }
    }
}
