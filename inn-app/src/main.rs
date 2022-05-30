use yew::prelude::*;
mod components;
mod network;
mod pubsub;
use crate::components::{App, Home, Login};
fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::start_app::<App>();
}
