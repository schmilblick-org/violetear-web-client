use yew::{html, Component, ComponentLink, Html, Renderable, ShouldRender};

struct Model {}

enum Msg {
    Login,
    Register,
}

impl Component for Model {
    // Some details omitted. Explore the examples to see more.

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
            <body style="padding: 30px 30px; text-align: center;",>
                <div style="position: relative; display: inline-block; padding: 20px; width: auto; text-align: left;",>
                    <p>
                        <input style="padding: 5px; line-height: 20px;", type="text", placeholder="username",></input>
                    </p>
                    <p>
                        <input style="padding: 5px; line-height: 20px;", type="text", placeholder="password",></input>
                    </p>
                    <div style="position: absolute; bottom: 0; left: 20px;",>
                        <button onclick=|_| Msg::Register,>{ "Register" }</button>
                    </div>
                    <div style="position: absolute; bottom: 0; right: 20px;",>
                        <button onclick=|_| Msg::Login,>{ "Login" }</button>
                    </div>
                </div>
            </body>
        }
    }
}

fn main() {
    yew::start_app::<Model>();
}