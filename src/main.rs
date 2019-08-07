#![recursion_limit = "8192"]

use std::collections::HashSet;

use failure::{Error, format_err};
use serde_derive::{Deserialize, Serialize};
use yew::{Component, ComponentLink, Html, html::ChangeData, Renderable, ShouldRender};
use yew::format::{Json, Nothing};
use yew::html;
use yew::services::console::ConsoleService;
use yew::services::fetch::{FetchService, FetchTask, Request, Response};
use yew::services::reader::{File, FileData, ReaderService, ReaderTask};
use yew::services::storage::{Area, StorageService};
use yew::virtual_dom::VNode;

const KEY: &str = "violetear.web-client.database";

struct Model {
    link: ComponentLink<Model>,
    storage_service: StorageService,
    fetch_service: FetchService,
    console_service: ConsoleService,
    reader_service: ReaderService,
    ft: Option<FetchTask>,
    config: Option<Config>,
    state: State,
    scene: Scene,
    loginregister_error: Option<String>,
    loginregister_form: LoginRegisterFormData,
    logout_error: Option<String>,
    fetched_profiles: Option<ProfilesResponse>,
    fetch_profiles_error: Option<String>,
    enabled_profiles: HashSet<String>,
    is_register_disabled: bool,
    is_register_loading: bool,
    is_login_loading: bool,
    is_login_disabled: bool,
    is_logout_loading: bool,
    is_logout_disabled: bool,
    rt: Option<ReaderTask>,
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
    LoginRegisterFormDataChange(LoginRegisterFormDataField, String),
    Login,
    LoginDone(Result<LoginResponse, Error>),
    Register,
    RegisterDone(Result<RegisterResponse, Error>),
    Logout,
    LogoutDone(Result<(), Error>),
    FetchProfiles,
    FetchProfilesDone(Result<ProfilesResponse, Error>),
    ToggleProfile(String),
    LoadFile(ChangeData),
    CreateReport(FileData),
    CreateReportDone(Result<CreateResponse, Error>),
    NoOp,
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

#[derive(Deserialize)]
pub struct Profile {
    pub id: i64,
    pub machine_name: String,
    pub human_name: String,
    pub module: String,
    pub config: Option<serde_json::Value>,
}

#[derive(Deserialize)]
pub struct ProfilesResponse {
    profiles: Vec<Profile>,
}

#[derive(Deserialize)]
pub struct CreateResponse {
    report_id: i64,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, mut link: ComponentLink<Self>) -> Self {
        let storage_service = StorageService::new(Area::Local);

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
            reader_service: ReaderService::new(),
            storage_service,
            scene: Scene::Loading,
            ft: None,
            rt: None,
            config: None,
            loginregister_error: None,
            loginregister_form: LoginRegisterFormData::default(),
            logout_error: None,
            fetched_profiles: None,
            fetch_profiles_error: None,
            enabled_profiles: HashSet::new(),
            is_register_disabled: false,
            is_register_loading: false,
            is_login_loading: false,
            is_login_disabled: false,
            is_logout_loading: false,
            is_logout_disabled: false,
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
                                    Msg::FetchConfigDone(Err(format_err!(
                                        "{}: could not fetch /config.json",
                                        meta.status
                                    )))
                                }
                            },
                        ),
                    ));
                false
            }
            Msg::FetchConfigDone(Ok(response)) => {
                self.config = Some(response);

                self.console_service
                    .log(&format!("Configuration was fetched.\n{:#?}", self.config));

                if self.state.token.is_some() {
                    self.link.send_self(Msg::FetchProfiles);
                    self.scene = Scene::Loading;
                } else {
                    self.scene = Scene::LoginRegister;
                }
                true
            }
            Msg::FetchConfigDone(Err(_)) => {
                self.scene = Scene::FetchConfigError;
                true
            }
            Msg::Login => {
                self.loginregister_error = None;
                self.is_register_disabled = true;
                self.is_login_loading = true;
                self.is_login_disabled = true;

                if let Some(config) = &self.config {
                    self.ft = Some(
                        self.fetch_service.fetch(
                            Request::builder()
                                .method("POST")
                                .uri(&format!("{}/v1/auth/login", config.api_url))
                                .header("Content-Type", "application/json")
                                .body(Json(&self.loginregister_form))
                                .unwrap(),
                            self.link.send_back(
                                move |response: Response<Json<Result<LoginResponse, Error>>>| {
                                    let (meta, Json(data)) = response.into_parts();
                                    if meta.status.is_success() {
                                        Msg::LoginDone(data)
                                    } else {
                                        Msg::LoginDone(Err(format_err!(
                                            "{}: could not login",
                                            meta.status
                                        )))
                                    }
                                },
                            ),
                        ),
                    );
                };
                true
            }
            Msg::LoginDone(Ok(login_response)) => {
                self.state.token = Some(login_response.token.unwrap());
                self.storage_service.store(KEY, Json(&self.state));
                self.is_register_disabled = false;
                self.is_login_loading = false;
                self.is_login_disabled = false;
                self.link.send_self(Msg::FetchProfiles);
                true
            }
            Msg::LoginDone(Err(_)) => {
                self.is_register_disabled = false;
                self.is_login_loading = false;
                self.is_login_disabled = false;
                self.loginregister_error = Some("Could not login".into());
                true
            }
            Msg::Register => {
                self.loginregister_error = None;
                self.is_register_disabled = true;
                self.is_register_loading = true;
                self.is_login_disabled = true;

                if let Some(config) = &self.config {
                    self.ft = Some(
                        self.fetch_service.fetch(
                            Request::builder()
                                .method("POST")
                                .uri(&format!("{}/v1/auth/register", config.api_url))
                                .header("Content-Type", "application/json")
                                .body(Json(&self.loginregister_form))
                                .unwrap(),
                            self.link.send_back(
                                move |response: Response<Json<Result<RegisterResponse, Error>>>| {
                                    let (meta, Json(data)) = response.into_parts();
                                    if meta.status.is_success() {
                                        Msg::RegisterDone(data)
                                    } else {
                                        Msg::RegisterDone(Err(format_err!(
                                            "{}: could not register",
                                            meta.status
                                        )))
                                    }
                                },
                            ),
                        ),
                    );
                };
                true
            }
            Msg::RegisterDone(Ok(register_response)) => {
                self.is_register_disabled = false;
                self.is_register_loading = false;
                self.is_login_disabled = false;
                self.state.token = Some(register_response.token.unwrap());
                self.storage_service.store(KEY, Json(&self.state));
                self.link.send_self(Msg::FetchProfiles);
                true
            }
            Msg::RegisterDone(Err(_)) => {
                self.is_register_disabled = false;
                self.is_register_loading = false;
                self.is_login_disabled = false;
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
                    self.logout_error = None;
                    self.is_logout_disabled = true;
                    self.is_logout_loading = true;

                    self.ft = Some(
                        self.fetch_service.fetch(
                            Request::builder()
                                .method("POST")
                                .uri(&format!("{}/v1/auth/logout", config.api_url))
                                .header("Content-Type", "application/json")
                                .header(
                                    "Authorization",
                                    self.state.token.as_ref().unwrap().to_owned(),
                                )
                                .body(Nothing)
                                .unwrap(),
                            self.link.send_back(move |response: Response<Nothing>| {
                                let (meta, _) = response.into_parts();
                                if meta.status.is_success() {
                                    Msg::LogoutDone(Ok(()))
                                } else {
                                    Msg::LogoutDone(Err(format_err!(
                                        "{}: could not logout",
                                        meta.status
                                    )))
                                }
                            }),
                        ),
                    );
                };
                true
            }
            Msg::LogoutDone(Ok(_)) => {
                self.is_logout_disabled = false;
                self.is_logout_loading = false;
                self.state.token = None;
                self.storage_service.store(KEY, Json(&self.state));
                self.loginregister_error = None;
                self.scene = Scene::LoginRegister;
                true
            }
            Msg::LogoutDone(Err(_)) => {
                self.is_logout_disabled = false;
                self.is_logout_loading = false;
                self.logout_error = Some("Could not logout".into());
                true
            }
            Msg::FetchProfiles => {
                if let Some(config) = &self.config {
                    self.ft = Some(
                        self.fetch_service.fetch(
                            Request::builder()
                                .method("GET")
                                .uri(&format!("{}/v1/profiles", config.api_url))
                                .header(
                                    "Authorization",
                                    self.state.token.as_ref().unwrap().to_owned(),
                                )
                                .body(Nothing)
                                .unwrap(),
                            self.link.send_back(
                                move |response: Response<Json<Result<ProfilesResponse, Error>>>| {
                                    let (meta, Json(profiles)) = response.into_parts();
                                    if meta.status.is_success() {
                                        Msg::FetchProfilesDone(profiles)
                                    } else {
                                        Msg::FetchProfilesDone(Err(format_err!(
                                            "{}: could not fetch profiles",
                                            meta.status
                                        )))
                                    }
                                },
                            ),
                        ),
                    );
                };
                true
            }
            Msg::FetchProfilesDone(Err(_)) => {
                self.fetch_profiles_error = Some("Could not fetch profiles".into());
                true
            }
            Msg::FetchProfilesDone(Ok(profiles_response)) => {
                self.scene = Scene::LoggedIn;

                self.enabled_profiles.clear();
                for profile in profiles_response.profiles.iter() {
                    self.enabled_profiles
                        .insert(profile.machine_name.to_owned());
                }

                self.fetched_profiles = Some(profiles_response);
                true
            }
            Msg::ToggleProfile(machine_name) => {
                if self.enabled_profiles.contains(&machine_name) {
                    self.enabled_profiles.remove(&machine_name);
                } else {
                    self.enabled_profiles.insert(machine_name.to_owned());
                }

                false
            }
            Msg::LoadFile(ChangeData::Files(ref file_list)) if file_list.len() == 1 => {
                let file = file_list.into_iter().next().unwrap();

                self.rt = Some(
                    self.reader_service
                        .read_file(file, self.link.send_back(Msg::CreateReport)),
                );

                false
            }
            Msg::CreateReport(file_data) => {
                if let Some(config) = &self.config {
                    self.ft = Some(
                        self.fetch_service.fetch_binary(
                            Request::builder()
                                .method("POST")
                                .uri(&format!(
                                    "{}/v1/reports/create?profiles={}",
                                    config.api_url,
                                    self.enabled_profiles
                                        .clone()
                                        .into_iter()
                                        .collect::<Vec<String>>()
                                        .join(",")
                                ))
                                .header(
                                    "Authorization",
                                    self.state.token.as_ref().unwrap().to_owned(),
                                )
                                .body(Ok(file_data.content))
                                .unwrap(),
                            self.link.send_back(
                                move |response: Response<Json<Result<CreateResponse, Error>>>| {
                                    let (meta, Json(response)) = response.into_parts();
                                    if meta.status.is_success() {
                                        Msg::CreateReportDone(response)
                                    } else {
                                        Msg::CreateReportDone(Err(format_err!(
                                            "{}: could not create report",
                                            meta.status
                                        )))
                                    }
                                },
                            ),
                        ),
                    );
                };

                false
            }
            Msg::CreateReportDone(Ok(create_response)) => {
                false
            }
            Msg::CreateReportDone(Err(_)) => {
                false
            }
            Msg::NoOp => false,
            _ => false,
        }
    }
}

impl Renderable<Model> for Model {
    fn view(&self) -> Html<Self> {
        match self.scene {
            Scene::Loading => html! {
                <section class="hero is-fullheight">
                    <div class="hero-body">
                        <div class="container">
                            <div class="columns is-centered is-vcentered is-mobile">
                                <div class="column" style="max-width: 250px;">
                                    <progress class="progress is-medium is-dark" max="100" />
                                </div>
                            </div>
                        </div>
                    </div>
                </section>
            },
            Scene::LoginRegister => html! {
                <section class="hero is-fullheight">
                    <div class="hero-body">
                        <div class="container">
                            <div class="columns is-centered is-vcentered is-mobile">
                                <div class="column" style="max-width: 300px;">
                                    {
                                        if let Some(error) = &self.loginregister_error {
                                            html! {
                                                <p class="has-text-centered" style="margin-top: 1em; margin-bottom: 1em;">
                                                    <span class="icon has-text-danger">
                                                        <i class="fas fa-info-circle"></i>
                                                    </span>
                                                    { error }
                                                </p>
                                            }
                                        } else {
                                            html! {
                                                <p class="has-text-centered" style="margin-top: 1em; margin-bottom: 1em;">
                                                    <span class="icon has-text-info">
                                                        <i class="fas fa-info-circle"></i>
                                                    </span>
                                                    { "Fill the form below" }
                                                </p>
                                            }
                                        }
                                    }
                                    <div class="box is-centered">
                                        <div class="field">
                                            <div class="control has-icons-left">
                                                <input class="input" type="text" placeholder="Username"
                                                    oninput=|e| Msg::LoginRegisterFormDataChange(LoginRegisterFormDataField::Username, e.value) />
                                                <span class="icon is-small is-left">
                                                    <i class="fas fa-user" />
                                                </span>
                                            </div>
                                        </div>
                                        <div class="field">
                                            <div class="control has-icons-left">
                                                <input class="input" type="password" placeholder="Password"
                                                    oninput=|e| Msg::LoginRegisterFormDataChange(LoginRegisterFormDataField::Password, e.value) />
                                                <span class="icon is-small is-left">
                                                    <i class="fas fa-lock" />
                                                </span>
                                            </div>
                                        </div>
                                        <div class="level is-mobile">
                                            <div class="level-left">
                                                <div class="level-item">
                                                    <div class="field">
                                                        <button class=if self.is_register_loading { "button is-loading" } else { "button" } type="button"
                                                            disabled=self.is_register_disabled
                                                            onclick=|_| Msg::Register>
                                                            { "Register" }
                                                        </button>
                                                    </div>
                                                </div>
                                            </div>
                                            <div class="level-right">
                                                <div class="level-item">
                                                    <div class="field">
                                                        <button class=if self.is_login_loading { "button is-loading" } else { "button" } type="button"
                                                            disabled=self.is_login_disabled
                                                            onclick=|_| Msg::Login>
                                                            { "Login" }
                                                        </button>
                                                    </div>
                                                </div>
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                </section>
            },
            Scene::FetchConfigError => html! {
                <section class="hero is-fullheight">
                    <div class="hero-body">
                        <div class="container">
                            <div class="columns is-centered is-vcentered is-mobile">
                                <div class="column">
                                    <div class="has-text-centered">
                                        <span class="icon has-text-danger">
                                            <i class="fas fa-info-circle" />
                                        </span>
                                        { "Could not fetch configuration, please reload to try again." }
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                </section>
            },
            Scene::LoggedIn => html! {
                <section class="hero is-fullheight">
                    <div class="hero-body">
                        <div class="container">
                            <div class="columns is-centered is-vcentered is-mobile">
                                <div class="column" style="max-width: 300px;">
                                    <nav class="panel">
                                        <p class="panel-heading">
                                            { "Profiles" }
                                        </p>
                                        {
                                            for self.fetched_profiles.iter().next().unwrap().profiles.iter().map(|profile| {
                                                html! {
                                                    <label class="panel-block is-unselectable">
                                                        <input
                                                            type="checkbox"
                                                            checked=true
                                                            value=&profile.machine_name.to_string()
                                                            onchange=|e| {
                                                                if let ChangeData::Value(value) = e {
                                                                    Msg::ToggleProfile(value)
                                                                } else {
                                                                    Msg::NoOp
                                                                }
                                                            }
                                                        />
                                                        { &profile.human_name }
                                                    </label>
                                                }
                                            })
                                        }
                                    </nav>

                                    <div class="file is-boxed is-centered">
                                        <label class="file-label">
                                            <input class="file-input" type="file" onchange=|e| Msg::LoadFile(e) />
                                            <span class="file-cta">
                                                <span class="file-icon">
                                                    <i class="fas fa-upload"></i>
                                                </span>
                                                <span class="file-label">
                                                    { "Drag to scan" }
                                                </span>
                                            </span>
                                        </label>
                                    </div>

                                    <div class="has-text-centered" style="margin-top: 2em; margin-bottom: 2em;">
                                        <button class=format!("button {} {}",
                                            if self.is_logout_loading { "is-loading" } else {""},
                                            if self.logout_error.is_some() {"is-danger"} else {""}),
                                            type="button"
                                            disabled=self.is_logout_disabled
                                            onclick=|_| Msg::Logout>
                                            { "Logout" }
                                        </button>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                </section>
            },
        }
    }
}

fn main() {
    yew::start_app::<Model>();
}
