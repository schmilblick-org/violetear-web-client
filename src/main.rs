use failure::Error;
use serde_derive::{Deserialize, Serialize};
use yew::format::{Json, Nothing};
use yew::services::fetch::{FetchService, FetchTask, Request, Response};
use yew::services::storage::{Area, StorageService};
use yew::services::console::ConsoleService;

use yew::{html, Component, ComponentLink, Html, Renderable, ShouldRender};

const KEY: &str = "violetear.web-client.database";

struct Model {
    link: ComponentLink<Model>,
    storage_service: StorageService,
    fetch_service: FetchService,
    console_service: ConsoleService,
    ft: Option<FetchTask>,
    config: Option<Config>,
    state: State,
    scene: Scene,
    loginregister_error: Option<String>,
    loginregister_form: LoginRegisterFormData,
}

enum Scene {
    Loading,
    LoginRegister,
    FetchConfigError,
}

enum Msg {
    FetchConfig,
    FetchConfigDone(Result<Config, Error>),
    FetchConfigError,
    Login,
    Register,
}

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    api_url: String,
}

#[derive(Serialize, Deserialize)]
struct State {
    token: Option<String>,
}

#[derive(Serialize, Default)]
struct LoginRegisterFormData {
    username: String,
    password: String,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let mut storage_service = StorageService::new(Area::Local);

        let state = {
            if let Json(Ok(state)) = storage_service.restore(KEY) {
                state
            } else {
                State { token: None }
            }
        };

        Self {
            link,
            state,
            fetch_service: FetchService::new(),
            console_service: ConsoleService::new(),
            ft: None,
            storage_service,
            config: None,
            scene: Scene::Loading,
            loginregister_error: None,
            loginregister_form: LoginRegisterFormData::default(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::FetchConfig => {
                self.ft =
                    Some(self.fetch_service.fetch(
                        Request::get("/config.json").body(Nothing).unwrap(),
                        self.link.send_back(
                            move |response: Response<Json<Result<Config, Error>>>| {
                                let (meta, Json(data)) = response.into_parts();
                                if meta.status.is_success() {
                                    Msg::FetchConfigDone(data)
                                } else {
                                    Msg::FetchConfigError
                                }
                            },
                        ),
                    ));
                false
            }
            Msg::FetchConfigDone(response) => {
                self.config = response.ok();

                self.console_service
                    .log(&format!("Configuration was fetched.\n{:#?}", self.config));

                self.scene = Scene::LoginRegister;
                true
            }
            Msg::FetchConfigError => {
                self.scene = Scene::FetchConfigError;
                true
            }
            Msg::Login => {
                if let Some(config) = &self.config {
                    self.ft = Some(
                        self.fetch_service.fetch(
                            Request::post(&format!("{}/login", config.api_url))
                                .body(Json(&self.loginregister_form))
                                .unwrap(),
                            self.link.send_back(
                                move |response: Response<Json<Result<Config, Error>>>| {
                                    let (meta, Json(data)) = response.into_parts();
                                    if meta.status.is_success() {
                                        Msg::FetchConfigDone(data)
                                    } else {
                                        Msg::FetchConfigError
                                    }
                                },
                            ),
                        ),
                    )
                };

                false
            }
            Msg::Register => false,
        }
    }
}

impl Renderable<Model> for Model {
    fn view(&self) -> Html<Self> {
        match self.scene {
            Scene::Loading => html! {
                <body onmouseover=|_| Msg::FetchConfig,> /* FIXME: Find a way to propagate a startup event */
                    <h3 style="text-align: center;",>
                        { "Application is loading.." }
                    </h3>
                </body>
            },
            Scene::LoginRegister => html! {
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

                            <p>
                                {
                                    if let Some(msg) = &self.loginregister_error {
                                        &msg
                                    } else {
                                        ""
                                    }
                                }
                            </p>
                    </div>
                </body>
            },
            Scene::FetchConfigError => html! {
                <body>
                    <h3 style="text-align: center;",>
                        { "Application configuration could not be loaded,
                            please reload the page to try again." }
                    </h3>
                </body>
            },
        }
    }
}

fn main() {
    yew::start_app::<Model>();
}