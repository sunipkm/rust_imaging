mod components;
mod connection;
mod selectors;

use once_cell::sync::Lazy;
use std::sync::{Arc, Mutex};

use ccdi_common::*;
use ccdi_imager_interface::ExposureArea;
use components::shooting_details::ShootingDetails;
use connection::ConnectionService;
use gloo::console;

use yew::html::Scope;
use yew::{html, Component, Context, Html};

use components::camera::CameraDetail;
use components::footer::Footer;
use components::menu::{Menu, MenuItem};
use components::status_bar::StatusBar;
use selectors::composition::CompositionDetail;
use selectors::picture::Picture;
use selectors::rendering::RenderingSelector;

use crate::components::system::System;
use crate::selectors::bool::BoolSelector;
use crate::selectors::float::FloatSelector;
use crate::selectors::shooting::ShootingDetail;
use crate::selectors::usize::UsizeInput;

// ============================================ PUBLIC =============================================

pub enum Msg {
    RegisterConnectionService(Scope<ConnectionService>),
    ConnectionState(ConnectionState),
    MessageReceived(ClientMessage),
    SendMessage(StateMessage),
    Action(UserAction),
    ParamUpdate(CameraParamMessage),
}

pub enum UserAction {
    MenuClick(MenuItem),
}

struct RoundSize {
    width: usize,
    height: usize,
}

pub struct Main {
    pub image: Option<Arc<RgbImage<u16>>>,
    pub view_state: ViewState,
    pub connection_state: ConnectionState,
    pub connection_context: Option<Scope<ConnectionService>>,
    pub selected_menu: MenuItem,
    pub x: Arc<Mutex<usize>>,
    pub y: Arc<Mutex<usize>>,
    pub w: Arc<Mutex<usize>>,
    pub h: Arc<Mutex<usize>>,
}

impl Main {
    fn receive_message(&mut self, message: ClientMessage) -> bool {
        match message {
            ClientMessage::Reconnect => {} // handled elsewhere
            ClientMessage::View(view) => self.view_state = view,
            ClientMessage::RgbImage(image) => self.image = Some(image),
        }

        true
    }

    fn render_tool(&self, ctx: &Context<Self>) -> Html {
        match self.selected_menu {
            MenuItem::Composition => self.render_composition(ctx),
            MenuItem::Cooling => self.render_cooling(ctx),
            MenuItem::Shoot => self.render_shoot(ctx),
            MenuItem::Info => html! {
                <CameraDetail data={self.view_state.camera_properties.clone()} />
            },
            MenuItem::System => self.render_system(ctx),
        }
    }

    fn render_composition(&self, ctx: &Context<Self>) -> Html {
        static FIRST_CALL: Lazy<Arc<Mutex<bool>>> = Lazy::new(|| Arc::new(Mutex::new(true)));
        let action = ctx
            .link()
            .callback(|action: StateMessage| Msg::SendMessage(action));

        let gain_changed = ctx
            .link()
            .callback(|gain: f64| Msg::ParamUpdate(CameraParamMessage::SetGain(gain as u16)));

        let time_changed = ctx
            .link()
            .callback(|time: f64| Msg::ParamUpdate(CameraParamMessage::SetTime(time)));

        let rendering_changed = ctx.link().callback(|value: RenderingType| {
            Msg::ParamUpdate(CameraParamMessage::SetRenderingType(value))
        });

        let autoexp_changed = ctx
            .link()
            .callback(|value: bool| Msg::ParamUpdate(CameraParamMessage::SetAutoExp(value)));

        let flipx_changed = ctx
            .link()
            .callback(|value: bool| Msg::ParamUpdate(CameraParamMessage::SetFlipX(value)));

        let flipy_changed = ctx
            .link()
            .callback(|value: bool| Msg::ParamUpdate(CameraParamMessage::SetFlipY(value)));

        let x = self.x.clone();
        let y = self.y.clone();
        let w = self.w.clone();
        let h = self.h.clone();

        {
            let mut cond = FIRST_CALL.lock().unwrap();
            if *cond {
                let roi = self
                    .view_state
                    .camera_properties
                    .clone()
                    .map(|prop| prop.basic.roi)
                    .unwrap_or(ExposureArea {
                        x: 0,
                        y: 0,
                        width: 0,
                        height: 0,
                    });

                *x.lock().unwrap() = roi.x;
                *y.lock().unwrap() = roi.y;
                *w.lock().unwrap() = roi.width;
                *h.lock().unwrap() = roi.height;

                *cond = false;
            }
        }

        let x_ = self.x.clone();
        let y_ = self.y.clone();
        let w_ = self.w.clone();
        let h_ = self.h.clone();

        let xp = self.x.clone();
        let yp = self.y.clone();
        let wp = self.w.clone();
        let hp = self.h.clone();

        let roi_changed = ctx.link().callback(move |_| {
            let value = (
                *x_.clone().lock().unwrap(),
                *y_.clone().lock().unwrap(),
                *w_.clone().lock().unwrap(),
                *h_.clone().lock().unwrap(),
            );
            *FIRST_CALL.clone().lock().unwrap() = true;
            Msg::ParamUpdate(CameraParamMessage::SetRoi(value))
        });

        let roi = self
            .view_state
            .camera_properties
            .clone()
            .map(|prop| prop.basic.roi)
            .unwrap_or(ExposureArea {
                x: 0,
                y: 0,
                width: 0,
                height: 0,
            });

        let refresh_roi = move |_| {
            *xp.lock().unwrap() = roi.x;
            *yp.lock().unwrap() = roi.y;
            *wp.lock().unwrap() = roi.width;
            *hp.lock().unwrap() = roi.height;
        };

        let exposure = self
            .view_state
            .camera_properties
            .clone()
            .map(|prop| prop.basic.exposure)
            .unwrap_or(0 as f32);

        let exposure_str = {
            if exposure < 0.001 {
                format!("{:5.2} us", (exposure * 1000000.0))
            } else if exposure < 1.0 {
                format!("{:5.2} ms", (exposure * 1000.0))
            } else {
                format!("{:.2} s", exposure)
            }
        };

        let csize = {
            if let Some(prop) = &self.view_state.camera_properties {
                let (width, height) = (prop.basic.width, prop.basic.height);
                RoundSize {
                    width: width.to_string().len(),
                    height: height.to_string().len(),
                }
            } else {
                RoundSize {
                    width: 1,
                    height: 1,
                }
            }
        };

        html! {
            <div>
                <BoolSelector
                    name = "Autoexposure"
                    selected_value = {self.view_state.camera_params.autoexp}
                    value_changed = {autoexp_changed}
                />
                <p>{"Current Exposure: "}{exposure_str}</p>
                <div style="border: 2px solid white;">
                    <p><b>{"Region of Interest"}</b></p>
                    <div class="float-container">
                        <div class="float-child">
                            <BoolSelector
                                name = "Flip X"
                                selected_value = {self.view_state.camera_params.flipx}
                                value_changed = {flipx_changed}
                            />
                        </div>
                        <div class="float-child">
                            <BoolSelector
                                name = "Flip Y"
                                selected_value = {self.view_state.camera_params.flipy}
                                value_changed = {flipy_changed}
                            />
                        </div>
                    </div>
                    <br/>
                    <p> {"Origin: X "}
                        <UsizeInput
                            value={*x.lock().unwrap()}
                            width={csize.width}
                            on_change={move |value| {*x.lock().unwrap() = value}}
                        />
                        {" Y "}
                        <UsizeInput
                        value={*y.lock().unwrap()}
                        width={csize.height}
                        on_change={move |value| {*y.lock().unwrap() = value}}
                        />
                    </p>
                    <p> {"Size: "}
                        <UsizeInput
                        value={*w.lock().unwrap()}
                        width={csize.width}
                        on_change={move |value| {*w.lock().unwrap() = value}}
                        />
                        {" x "}
                        <UsizeInput
                        value={*h.lock().unwrap()}
                        width={csize.height}
                        on_change={move |value| {*h.lock().unwrap() = value}}
                        />
                    </p>

                    <button onclick={roi_changed}>{"Update ROI"}</button>
                    <button onclick={refresh_roi}>{"Refresh ROI"}</button>
                </div>
                <FloatSelector
                    name="Set camera gain"
                    config={self.view_state.config.gain.clone()}
                    selected_value={self.view_state.camera_params.gain as f64}
                    value_changed={gain_changed}
                />
                <FloatSelector
                    name="Set camera exposure time"
                    config={self.view_state.config.exposure.clone()}
                    selected_value={self.view_state.camera_params.time}
                    value_changed={time_changed}
                />
                <RenderingSelector
                    rendering_changed={rendering_changed}
                    selected_value={self.view_state.camera_params.rendering}
                />
                <CompositionDetail
                    on_action={action}
                    camera_params={self.view_state.camera_params.clone()}
                />
            </div>
        }
    }

    fn render_cooling(&self, ctx: &Context<Self>) -> Html {
        let cooling_changed = ctx
            .link()
            .callback(|temp: f64| Msg::ParamUpdate(CameraParamMessage::SetTemp(temp)));

        let heating_changed = ctx
            .link()
            .callback(|temp: f64| Msg::ParamUpdate(CameraParamMessage::SetHeatingPwm(temp)));

        html! {
            <div>
                <p>{"Chip temperature: "}
                {
                    self.view_state.camera_properties
                        .clone()
                        .map(|prop| prop.basic.temperature.to_string())
                        .unwrap_or(String::from("?"))
                }
                {
                    " C"
                }
                </p>
                <FloatSelector
                    name="Camera Cooling"
                    config={self.view_state.config.cooling.clone()}
                    selected_value={self.view_state.camera_params.temperature}
                    value_changed={cooling_changed}
                />
                <FloatSelector
                    name="Telescope Heating PWM"
                    config={self.view_state.config.heating.clone()}
                    selected_value={self.view_state.camera_params.heating_pwm}
                    value_changed={heating_changed}
                />
            </div>
        }
    }

    fn render_shoot(&self, ctx: &Context<Self>) -> Html {
        let action = ctx
            .link()
            .callback(|action: StateMessage| Msg::SendMessage(action));

        html! {
            <div>
                <ShootingDetail
                    on_action={action.clone()}
                    storage_details={self.view_state.storage_detail.clone()}
                />

                <CompositionDetail
                    on_action={action}
                    camera_params={self.view_state.camera_params.clone()}
                />
            </div>
        }
    }

    fn render_system(&self, ctx: &Context<Self>) -> Html {
        let action = ctx
            .link()
            .callback(|action: StateMessage| Msg::SendMessage(action));

        html! {
            <System on_action={action.clone()}/>
        }
    }

    fn render_main(&self) -> Html {
        match self.selected_menu {
            MenuItem::Shoot => html! {
                <ShootingDetails storage_details={self.view_state.storage_detail.clone()} />
            },
            _ => html! {
                <Picture
                    image={self.image.clone()}
                    hist_width={self.view_state.config.histogram_width}
                    hist_height={self.view_state.config.histogram_height}
                />
            },
        }
    }
}

impl Component for Main {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        static X: Lazy<Arc<Mutex<usize>>> = Lazy::new(|| Arc::new(Mutex::new(0)));
        static Y: Lazy<Arc<Mutex<usize>>> = Lazy::new(|| Arc::new(Mutex::new(0)));
        static W: Lazy<Arc<Mutex<usize>>> = Lazy::new(|| Arc::new(Mutex::new(0)));
        static H: Lazy<Arc<Mutex<usize>>> = Lazy::new(|| Arc::new(Mutex::new(0)));

        Self {
            image: None,
            view_state: Default::default(),
            selected_menu: MenuItem::Composition,
            connection_state: ConnectionState::Disconnected,
            connection_context: None,
            x: X.clone(),
            y: Y.clone(),
            w: W.clone(),
            h: H.clone(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SendMessage(message) => {
                match self.connection_context.as_ref() {
                    None => console::warn!("No connection service registered."),
                    Some(context) => context.send_message(connection::Msg::SendData(message)),
                }
                false
            }
            Msg::RegisterConnectionService(context) => {
                self.connection_context = Some(context);
                false
            }
            Msg::ConnectionState(state) => {
                self.connection_state = state;
                true
            }
            Msg::ParamUpdate(message) => {
                ctx.link()
                    .send_message(Msg::SendMessage(StateMessage::CameraParam(message)));
                false
            }
            Msg::Action(action) => {
                match action {
                    UserAction::MenuClick(menuitem) => {
                        self.selected_menu = menuitem;
                    }
                }
                true
            }
            Msg::MessageReceived(message) => self.receive_message(message),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let menu_clicked = ctx
            .link()
            .callback(|action: MenuItem| Msg::Action(UserAction::MenuClick(action)));

        let client_message_received = ctx
            .link()
            .callback(|message: ClientMessage| Msg::MessageReceived(message));

        let connection_state_changed = ctx
            .link()
            .callback(|state: ConnectionState| Msg::ConnectionState(state));

        html! {
            <>
                <ConnectionService
                    on_message={client_message_received}
                    on_state_change={connection_state_changed}
                />
                <StatusBar
                    connection={self.connection_state}
                    logic={self.view_state.status.clone()}
                />
                <Menu clicked={menu_clicked} selected={self.selected_menu} />
                <div class="main-row">
                    <div class="main-image-column">
                        { self.render_main() }
                    </div>
                    <div class="main-tool-column">
                        { self.render_tool(ctx) }
                    </div>
                </div>
                <Footer text={self.view_state.detail.clone()}
                />
            </>
        }
    }
}

fn main() {
    yew::Renderer::<Main>::new().render();
}
