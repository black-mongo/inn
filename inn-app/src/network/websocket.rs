use futures::{channel::mpsc::Sender, SinkExt, StreamExt};
use reqwasm::websocket::{futures::WebSocket, Message};

use wasm_bindgen_futures::spawn_local;
use yew_agent::Dispatched;

use crate::pubsub::event_bus::{EventBus, Publish};

pub struct WebsocketService {
    pub tx: Sender<String>,
}

impl WebsocketService {
    pub fn new() -> Self {
        let ws = WebSocket::open("ws://127.0.0.1:8080").unwrap();

        let (mut write, mut read) = ws.split();

        let (in_tx, mut in_rx) = futures::channel::mpsc::channel::<String>(1000);
        let mut event_bus = EventBus::dispatcher();

        spawn_local(async move {
            while let Some(s) = in_rx.next().await {
                log::debug!("got event from channel! {}", s);
                write.send(Message::Text(s)).await.unwrap();
            }
        });

        spawn_local(async move {
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Text(data)) => {
                        log::debug!("from websocket: {}", data);
                        event_bus.send(Publish::EventBusMsg(data));
                    }
                    Ok(Message::Bytes(b)) => {
                        let decoded = std::str::from_utf8(&b);
                        if let Ok(val) = decoded {
                            log::debug!("from websocket: {}", val);
                            event_bus.send(Publish::EventBusMsg(val.into()));
                        }
                    }
                    Err(e) => {
                        log::error!("ws: {:?}", e)
                    }
                }
            }
            log::debug!("WebSocket Closed");
        });

        Self { tx: in_tx }
    }
}
