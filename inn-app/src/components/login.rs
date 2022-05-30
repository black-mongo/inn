use crate::components::app::Route;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_router::prelude::*;

use super::app::AppContext;
pub struct Login {
    r: NodeRef,
    user: String,
}
pub enum Msg {
    User(String),
    MouseOver,
}
impl Component for Login {
    type Message = Msg;
    type Properties = ();
    fn create(ctx: &Context<Self>) -> Self {
        Self {
            r: NodeRef::default(),
            user: "".to_owned(),
        }
    }
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::User(user) => self.user = user,
            Msg::MouseOver => {
                if let Some(input) = self.r.cast::<HtmlInputElement>() {
                    input.focus().unwrap();
                }
            }
        }
        true
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        let oninput = ctx.link().callback(|e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            log::debug!("oninput = {}", input.value());
            Msg::User(input.value())
        });
        let history = ctx.link().history().unwrap();
        let onclick = {
            let (app_ctx, _) = ctx
                .link()
                .context::<AppContext>(Callback::noop())
                .expect("context to be set");
            let username = self.user.clone();
            let app_ctx = app_ctx.clone();
            Callback::once(move |_| {
                log::debug!("onclick = {:?}", username.clone());
                *app_ctx.user.borrow_mut() = username;
                history.push(Route::Home);
            })
        };
        let onmouseover = ctx.link().callback(|_| Msg::MouseOver);
        html! {
           <div class="bg-gray-800 flex w-screen">
                <div class="container mx-auto flex flex-col justify-center items-center">
                    <form class="m-4 flex">
                        <input {oninput} ref = {self.r.clone()} onmouseover={onmouseover} class="rounded-l-lg p-4 border-t mr-0 border-b border-l text-gray-800 border-gray-200 bg-white" placeholder="Username" />
                        <Link<Route> to={Route::Home}> <button {onclick}
                         class="px-8 rounded-r-lg bg-violet-600 text-white font-bold p-4 uppercase border-violet-600 border-t border-b border-r" >{"Go Chatting!"}</button></Link<Route>>
                    </form>
                </div>
            </div>
        }
    }

    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        true
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {}

    fn destroy(&mut self, ctx: &Context<Self>) {}
}
