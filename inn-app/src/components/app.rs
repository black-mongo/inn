use std::{cell::RefCell, rc::Rc};

use crate::components::{Home, Login};
use yew::prelude::*;
use yew_router::prelude::*;
pub type AppContext = Rc<AppContextInner>;
#[derive(Clone, Debug, PartialEq)]
pub struct AppContextInner {
    pub user: RefCell<String>,
}
pub struct App {
    pub ctx: AppContext,
}
#[derive(Routable, PartialEq, Clone)]
pub enum Route {
    #[at("/home")]
    Home,
    #[at("/")]
    Login,
}
fn switch(route: &Route) -> Html {
    match route {
        Route::Home => html! {<Home />},
        Route::Login => html! {<Login/>},
    }
}
#[function_component(Header)]
fn header() -> Html {
    html! {
        <div>
          <h1>{"header"}</h1>
        </div>
    }
}
#[function_component(Footer)]
fn footer() -> Html {
    html! {
        <div>
         <h5>{"footer"}</h5>
        </div>
    }
}
impl Component for App {
    type Message = ();
    type Properties = ();
    fn create(ctx: &Context<Self>) -> Self {
        log::debug!("app create");
        Self {
            ctx: Rc::new(AppContextInner {
                user: RefCell::new("".to_string()),
            }),
        }
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
        <ContextProvider<AppContext> context={self.ctx.clone()}>
            <BrowserRouter>
                <div class="flex w-screen h-screen">
                    <Switch<Route> render={Switch::render(switch)}/>
                </div>
            </BrowserRouter>
        </ContextProvider<AppContext>>
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        false
    }

    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        true
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {}

    fn destroy(&mut self, ctx: &Context<Self>) {}
}
