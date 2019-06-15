use yew::{html, Component, ComponentLink, Html, Renderable, ShouldRender};

struct Model {}

enum Msg {
    Login,
    Register,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, _: ComponentLink<Self>) -> Self {
        Model {}
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Login => false,
            Msg::Register => false,
        }
    }
}

impl Renderable<Model> for Model {
    fn view(&self) -> Html<Self> {
        html! {
            <body class="login-body",>
                <div class="login-div",>
                        <input class="login-input", type="text", placeholder="username",/>
                        <input class="login-input", type="text", placeholder="password",/>

                        <button class="login-button", style="left: 20px", onclick=|_| Msg::Register,>
                            { "Register" }
                        </button>
                        <button class="login-button", style="right: 20px", onclick=|_| Msg::Login,>
                            { "Login" }
                        </button>
                </div>
            </body>
        }
    }
}

fn main() {
    yew::start_app::<Model>();
}