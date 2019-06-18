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
    logout_error: Option<String>,
}

enum Scene {
    Loading,
    LoginRegister,
    FetchConfigError,
    LoggedIn,
}

enum Msg {
    FetchConfig,
    FetchConfigDone(Result<Config, Error>),
    FetchConfigError,
    LoginRegisterFormDataChange(LoginRegisterFormDataField, String),
    Login,
    LoginDone(Result<LoginResponse, Error>),
    LoginError,
    Register,
    RegisterDone(Result<RegisterResponse, Error>),
    RegisterError,
    Logout,
    LogoutDone(Result<LogoutResponse, Error>),
    LogoutError,
}

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    api_url: String,
}

#[derive(Serialize, Deserialize)]
struct State {
    token: Option<String>,
}

enum LoginRegisterFormDataField {
    Username,
    Password,
}

#[derive(Serialize, Default)]
struct LoginRegisterFormData {
    username: String,
    password: String,
}

#[derive(Deserialize)]
struct LoginResponse {
    token: Option<String>,
}

#[derive(Deserialize)]
struct RegisterResponse {
    token: Option<String>,
}

#[derive(Serialize)]
struct Logout {
    token: String,
}

#[derive(Deserialize)]
struct LogoutResponse {}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, mut link: ComponentLink<Self>) -> Self {
        let mut storage_service = StorageService::new(Area::Local);

        let state = {
            if let Json(Ok(state)) = storage_service.restore(KEY) {
                state
            } else {
                State { token: None }
            }
        };

        link.send_self(Msg::FetchConfig);

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
            logout_error: None,
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

                self.scene = if self.state.token.is_some() {
                    Scene::LoggedIn
                } else {
                    Scene::LoginRegister
                };
                true
            }
            Msg::FetchConfigError => {
                self.scene = Scene::FetchConfigError;
                true
            }
            Msg::Login => {
                self.loginregister_error = None;

                if let Some(config) = &self.config {
                    self.ft = Some(
                        self.fetch_service.fetch(
                            Request::builder()
                                .method("POST")
                                .uri(&format!("{}/login", config.api_url))
                                .header("Content-Type", "application/json")
                                .body(Json(&self.loginregister_form))
                                .unwrap(),
                            self.link.send_back(
                                move |response: Response<Json<Result<LoginResponse, Error>>>| {
                                    let (meta, Json(data)) = response.into_parts();
                                    if meta.status.is_success() {
                                        Msg::LoginDone(data)
                                    } else {
                                        Msg::LoginError
                                    }
                                },
                            ),
                        ),
                    )
                };
                true
            }
            Msg::LoginDone(response) => {
                self.state.token = response
                    .map(|login_response| login_response.token.unwrap())
                    .ok();
                self.storage_service.store(KEY, Json(&self.state));
                self.scene = Scene::LoggedIn;
                true
            }
            Msg::LoginError => {
                self.loginregister_error = Some("Could not login".into());
                true
            }
            Msg::Register => {
                self.loginregister_error = None;

                if let Some(config) = &self.config {
                    self.ft = Some(
                        self.fetch_service.fetch(
                            Request::builder()
                                .method("POST")
                                .uri(&format!("{}/register", config.api_url))
                                .header("Content-Type", "application/json")
                                .body(Json(&self.loginregister_form))
                                .unwrap(),
                            self.link.send_back(
                                move |response: Response<Json<Result<RegisterResponse, Error>>>| {
                                    let (meta, Json(data)) = response.into_parts();
                                    if meta.status.is_success() {
                                        Msg::RegisterDone(data)
                                    } else {
                                        Msg::RegisterError
                                    }
                                },
                            ),
                        ),
                    )
                };
                true
            }
            Msg::RegisterDone(response) => {
                self.state.token = response
                    .map(|register_response| register_response.token.unwrap())
                    .ok();
                self.storage_service.store(KEY, Json(&self.state));
                self.scene = Scene::LoggedIn;
                true
            }
            Msg::RegisterError => {
                self.loginregister_error = Some("Could not register".into());
                true
            }
            Msg::LoginRegisterFormDataChange(field, value) => {
                match field {
                    LoginRegisterFormDataField::Username => {
                        self.loginregister_form.username = value;
                    }
                    LoginRegisterFormDataField::Password => {
                        self.loginregister_form.password = value;
                    }
                }
                false
            }
            Msg::Logout => {
                if let Some(config) = &self.config {
                    let logout = Logout {
                        token: self.state.token.as_ref().unwrap().to_owned(),
                    };

                    self.ft = Some(
                        self.fetch_service.fetch(
                            Request::builder()
                                .method("POST")
                                .uri(&format!("{}/logout", config.api_url))
                                .header("Content-Type", "application/json")
                                .body(Json(&logout))
                                .unwrap(),
                            self.link.send_back(
                                move |response: Response<Json<Result<LogoutResponse, Error>>>| {
                                    let (meta, Json(data)) = response.into_parts();
                                    if meta.status.is_success() {
                                        Msg::LogoutDone(data)
                                    } else {
                                        Msg::LogoutError
                                    }
                                },
                            ),
                        ),
                    )
                };
                false
            }
            Msg::LogoutDone(_response) => {
                self.state.token = None;
                self.storage_service.store(KEY, Json(&self.state));
                self.loginregister_error = None;
                self.scene = Scene::LoginRegister;
                true
            }
            Msg::LogoutError => {
                self.logout_error = Some("Could not logout".into());
                true
            }
            _ => false,
        }
    }
}

impl Renderable<Model> for Model {
    fn view(&self) -> Html<Self> {
        match self.scene {
            Scene::Loading => html! {
                <body>
                    <h3 style="text-align: center;",>
                        { "Application is loading.." }
                    </h3>
                </body>
            },
            Scene::LoginRegister => html! {
                <body class="login-body",>
                    <div class="login-div",>
                            <input class="login-input",
                                oninput=|e| {
                                    Msg::LoginRegisterFormDataChange(LoginRegisterFormDataField::Username, e.value)
                                },
                                type="text",
                                placeholder="username", />
                            <input class="login-input",
                                oninput=|e| {
                                    Msg::LoginRegisterFormDataChange(LoginRegisterFormDataField::Password, e.value)
                                },
                                type="text",
                                placeholder="password", />

                            <button class="login-button", style="left: 20px", onclick=|_| Msg::Register,>
                                { "Register" }
                            </button>
                            <button class="login-button", style="right: 20px", onclick=|_| Msg::Login,>
                                { "Login" }
                            </button>
                    </div>
                    <p>
                        {
                            if let Some(msg) = &self.loginregister_error {
                                &msg
                            } else {
                                ""
                            }
                        }
                    </p>
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
            Scene::LoggedIn => html! {
                <body class="login-body",>
                    <div class="login-div",>
                        <h3 style="text-align: center;",>
                            {
                                if let Some(token) = &self.state.token {
                                    format!("Logged in with token {}", token)
                                } else {
                                    String::new()
                                }
                            }
                        </h3>
                        <div style="text-align: center;",>
                            <button style="line-height: 20px;", onclick=|_| Msg::Logout,>
                                { "Logout" }
                            </button>
                        </div>
                    </div>
                    <p>
                        {
                            if let Some(msg) = &self.logout_error {
                                &msg
                            } else {
                                ""
                            }
                        }
                    </p>
                </body>
            },
        }
    }
}

fn main() {
    yew::start_app::<Model>();
}