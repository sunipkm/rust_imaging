mod components;
mod connection;
mod selectors;

use once_cell::sync::Lazy;
use std::sync::{Arc, Mutex};
use web_sys::console::log_1;

use ccdi_common::*;
use ccdi_imager_interface::ExposureArea;
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

use crate::components::system::System;
use crate::selectors::autoexp::AutoExpConfig;
use crate::selectors::bool::BoolSelector;
use crate::selectors::float::FloatSelector;
use crate::selectors::floatin::FloatInput;
use crate::selectors::shooting::ShootingDetail;
use crate::selectors::usize::UsizeInput;
// use crate::selectors::autoexp::AutoExpSelector;

use log::info;

// ============================================ PUBLIC =============================================

pub enum Msg {
    RegisterConnectionService(Scope<ConnectionService>),
    ConnectionState(ConnectionState),
    MessageReceived(ClientMessage),
    SendMessage(StateMessage),
    Action(UserAction),
    CParamUpdate(CameraParamMessage),
    IParamUpdate(ImageParamMessage),
}

pub enum UserAction {
    MenuClick(MenuItem),
}

pub struct Main {
    pub image: Option<Arc<Vec<u8>>>, // PNG Image data
    pub view_state: ViewState,
    pub connection_state: ConnectionState,
    pub connection_context: Option<Scope<ConnectionService>>,
    pub selected_menu: MenuItem,
    pub x: Arc<Mutex<usize>>,
    pub y: Arc<Mutex<usize>>,
    pub w: Arc<Mutex<usize>>,
    pub h: Arc<Mutex<usize>>,
    pub time: Arc<Mutex<String>>,
}

impl Main {
    // This is where incoming messages from the server are handled. ~Mit
    fn receive_message(&mut self, message: ClientMessage) -> bool {
        match message {
            ClientMessage::Reconnect => {} // handled elsewhere
            ClientMessage::View(view) => self.view_state = *view,
            ClientMessage::PngImage(image) => self.image = Some(image),
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
        static LAST_CALL_CAMERA: Lazy<Arc<Mutex<bool>>> = Lazy::new(|| Arc::new(Mutex::new(false)));
        let action = ctx
            .link()
            .callback(|action: StateMessage| Msg::SendMessage(action));

        let gain_changed = ctx.link().callback(|gain: f64| {
            // let gain = gain.parse::<f64>().unwrap_or(200.0);
            Msg::CParamUpdate(CameraParamMessage::SetGain(gain as u16))
        });

        let autoexp_changed = ctx
            .link()
            .callback(|value: bool| Msg::CParamUpdate(CameraParamMessage::SetAutoExp(value)));

        let flipx_changed = ctx
            .link()
            .callback(|value: bool| Msg::IParamUpdate(ImageParamMessage::SetFlipX(value)));

        let flipy_changed = ctx
            .link()
            .callback(|value: bool| Msg::IParamUpdate(ImageParamMessage::SetFlipY(value)));

        let x = self.x.clone();
        let y = self.y.clone();
        let w = self.w.clone();
        let h = self.h.clone();

        {
            let cond = self.view_state.status.camera == ConnectionState::Established;
            let mut last_cond = LAST_CALL_CAMERA.lock().unwrap();
            if cond && !*last_cond {
                *x.lock().unwrap() = self.view_state.image_params.x as usize;
                *y.lock().unwrap() = self.view_state.image_params.y as usize;
                *w.lock().unwrap() = self.view_state.image_params.w as usize;
                *h.lock().unwrap() = self.view_state.image_params.h as usize;
            }
            *last_cond = cond;
        }

        let x_ = self.x.clone();
        let y_ = self.y.clone();
        let w_ = self.w.clone();
        let h_ = self.h.clone();

        let time_cb = self.time.clone();
        let time = self.time.clone();

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
            let value = (
                value.0 as u16,
                value.1 as u16,
                value.2 as u16,
                value.3 as u16,
            );
            Msg::IParamUpdate(ImageParamMessage::SetRoi(value))
        });

        let exposure = self
            .view_state
            .camera_properties
            .clone()
            .map(|prop| prop.basic.exposure)
            .unwrap_or(0 as f32);

        // Callback called when we press the "Send Test Message" HTML button. ~Mit
        let client_test_message = ctx.link().callback(move |_| {
            // Possibly correct example of how to send a message to the server. ~Mit
            info!("Sending test message to server.");
            Msg::SendMessage(StateMessage::ClientInformation((
                "Hello".to_owned(),
                "Hello, server!".to_string(),
            )))
        });

        let time_changed_btn = ctx.link().callback(move |_| {
            let mut val = time_cb.lock().unwrap();
            let value = val.parse::<f64>();
            if let Ok(value) = value {
                if !(0.0..=3600.0).contains(&value) {
                    *val = format!("{:.6}", exposure);
                    return Msg::CParamUpdate(CameraParamMessage::SetTime(exposure as f64));
                }
                Msg::CParamUpdate(CameraParamMessage::SetTime(value))
            } else {
                *val = "0.001".to_string();
                Msg::CParamUpdate(CameraParamMessage::SetTime(0.001))
            }
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

        let exposure_str = {
            if exposure < 0.001 {
                format!("{:5.2} us", (exposure * 1000000.0))
            } else if exposure < 1.0 {
                format!("{:5.2} ms", (exposure * 1000.0))
            } else {
                format!("{:.2} s", exposure)
            }
        };

        html! {
            <div>
                <BoolSelector
                    name = "Autoexposure"
                    selected_value = {self.view_state.camera_params.autoexp}
                    value_changed = {autoexp_changed}
                />
                <p>{"Current Exposure: "}{exposure_str}{" Gain:"}{self.view_state.camera_params.gain.to_string()}</p>
                <div style="border: 2px solid white;">
                    <p><b>{"Region of Interest"}</b></p>
                    <div class="float-container">
                        <div class="float-child">
                            <BoolSelector
                                name = "Flip X"
                                selected_value = {self.view_state.image_params.flipx}
                                value_changed = {flipx_changed}
                            />
                        </div>
                        <div class="float-child">
                            <BoolSelector
                                name = "Flip Y"
                                selected_value = {self.view_state.image_params.flipy}
                                value_changed = {flipy_changed}
                            />
                        </div>
                    </div>
                    <br/>
                    <p> {"Origin:"}
                    <div class="div-table-row w100p">
                        <div class="div-table-col w5p"> {"X:"} </div>
                        <div class="div-table-col w40p">
                        <UsizeInput
                            value={*x.lock().unwrap()}
                            on_change={move |value| {*x.lock().unwrap() = value}}
                        />
                        </div>
                        <div class="div-table-col w5p"> {"Y:"} </div>
                        <div class="div-table-col w40p">
                        <UsizeInput
                            value={*y.lock().unwrap()}
                            on_change={move |value| {*y.lock().unwrap() = value}}
                        />
                        </div>
                    </div> // div-table-row w100p
                    </p>
                    <p> {"Size: "}
                    <div class="div-table-row w100p">
                    <div class="div-table-col w40p">
                        <UsizeInput
                            value={*w.lock().unwrap()}
                            on_change={move |value| {*w.lock().unwrap() = value}}
                        />
                    </div>
                    <div class="div-table-col w5p"> {"×"} </div>
                    <div class="div-table-col w40p">
                        <UsizeInput
                            value={*h.lock().unwrap()}
                            on_change={move |value| {*h.lock().unwrap() = value}}
                        />
                    </div>
                    </div> // div-table-row w100p
                    </p>

                    <button onclick={roi_changed}>{"Update ROI"}</button>
                    <button onclick={refresh_roi}>{"Refresh ROI"}</button>
                    </div>
                    <p>
                    <div class="div-table-row w100p">
                    <div class="div-table-col w30p"> {"Exposure"} </div>
                    <div class="div-table-col w50p">
                        <FloatInput
                            value={(*time.lock().unwrap()).clone()}
                            range={None}
                            sigfig={6}
                            on_change={move |value: f64| {
                                *time.lock().unwrap() = value.to_string();
                            }}
                        />
                    </div>
                    <div class="div-table-col w5p"> {"s"} </div>
                    </div> // div-table-row w100p
                    <div class="div-table-row w100p">
                    <div class="div-table-col w30p"> {"Gain"} </div>
                    <div class="div-table-col w50p">
                        <FloatInput
                            value={format!("{:.0}", self.view_state.camera_params.gain)}
                            range={None}
                            sigfig={0}
                            on_change={gain_changed}
                        />
                    </div>
                    </div> // div-table-row w100p
                    </p>
                <button onclick={time_changed_btn}>{"Update Exposure"}</button>
                <button onclick={client_test_message}>{"Send Test Message"}</button>
                <CompositionDetail
                    on_action={action.clone()}
                    image_params={self.view_state.image_params.clone()}
                    camera_params={self.view_state.camera_params.clone()}
                />
                <AutoExpConfig
                    on_action={action}
                    view_state={self.view_state.clone()}
                    image_params={self.view_state.image_params.clone()}
                />
            </div>
        }
    }

    fn render_cooling(&self, ctx: &Context<Self>) -> Html {
        let cooling_changed = ctx
            .link()
            .callback(|temp: f64| Msg::CParamUpdate(CameraParamMessage::SetTemp(temp)));

        let heating_changed = ctx
            .link()
            .callback(|temp: f64| Msg::CParamUpdate(CameraParamMessage::SetHeatingPwm(temp)));

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
                    image_params={self.view_state.image_params.clone()}
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
        html! {
            <Picture
                image={self.image.clone()}
                hist_width={self.view_state.config.histogram_width}
                hist_height={self.view_state.config.histogram_height}
                onresize={|val| log_1(&format!("Resized: {:?}", val).into())}
            />
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
        static T: Lazy<Arc<Mutex<String>>> = Lazy::new(|| Arc::new(Mutex::new("0".to_string())));

        Self {
            image: None,
            view_state: Default::default(),
            selected_menu: MenuItem::System,
            connection_state: ConnectionState::Disconnected,
            connection_context: None,
            x: X.clone(),
            y: Y.clone(),
            w: W.clone(),
            h: H.clone(),
            time: T.clone(),
        }
    }

    // Update parses outbound messages and acts accordingly; it also calls receive_message for inbound messages. ~Mit
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
            Msg::IParamUpdate(message) => {
                ctx.link()
                    .send_message(Msg::SendMessage(StateMessage::ImageParam(message)));
                false
            }
            Msg::CParamUpdate(message) => {
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
