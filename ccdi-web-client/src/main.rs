mod components;
mod connection;
mod selectors;

use std::sync::Arc;

use ccdi_common::*;
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

use crate::selectors::float::FloatSelector;
use crate::selectors::shooting::ShootingDetail;

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

pub struct Main {
    pub image: Option<Arc<RgbImage<u16>>>,
    pub view_state: ViewState,
    pub connection_state: ConnectionState,
    pub connection_context: Option<Scope<ConnectionService>>,
    pub selected_menu: MenuItem,
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
            MenuItem::Info => html!{
                <CameraDetail data={self.view_state.camera_properties.clone()} />
            },
        }
    }

    fn render_composition(&self, ctx: &Context<Self>) -> Html {
        let action = ctx.link()
            .callback(|action: StateMessage| Msg::SendMessage(action));

        let gain_changed = ctx.link().callback(
            |gain: f64| Msg::ParamUpdate(CameraParamMessage::SetGain(gain as u16))
        );

        let time_changed = ctx.link().callback(
            |time: f64| Msg::ParamUpdate(CameraParamMessage::SetTime(time))
        );

        let rendering_changed = ctx.link().callback(
            |value: RenderingType| Msg::ParamUpdate(CameraParamMessage::SetRenderingType(value))
        );

        html!{
            <div>
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
        let cooling_changed = ctx.link().callback(
            |temp: f64| Msg::ParamUpdate(CameraParamMessage::SetTemp(temp))
        );

        let heating_changed = ctx.link().callback(
            |temp: f64| Msg::ParamUpdate(CameraParamMessage::SetHeatingPwm(temp))
        );

        html!{
            <div>
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
        let action = ctx.link()
            .callback(|action: StateMessage| Msg::SendMessage(action));

        html!{
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

    fn render_main(&self) -> Html {
        match self.selected_menu {
            MenuItem::Shoot => html!{
                <ShootingDetails storage_details={self.view_state.storage_detail.clone()} />
            },
            _ => html!{
                <Picture
                    image={self.image.clone()}
                    hist_width={self.view_state.config.histogram_width}
                    hist_height={self.view_state.config.histogram_height}
                />
            }
        }
    }
}

impl Component for Main {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            image: None,
            view_state: Default::default(),
            selected_menu: MenuItem::Composition,
            connection_state: ConnectionState::Disconnected,
            connection_context: None,
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
            },
            Msg::ParamUpdate(message) => {
                ctx.link().send_message(Msg::SendMessage(StateMessage::CameraParam(message)));
                false
            }
            Msg::Action(action) => {
                match action {
                    UserAction::MenuClick(menuitem) => {
                        self.selected_menu = menuitem;
                    },
                }
                true
            }
            Msg::MessageReceived(message) => {
                self.receive_message(message)
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let menu_clicked = ctx.link().callback(
            |action: MenuItem| Msg::Action(UserAction::MenuClick(action))
        );

        let client_message_received = ctx.link().callback(
            |message: ClientMessage| Msg::MessageReceived(message)
        );

        let connection_state_changed = ctx.link().callback(
            |state: ConnectionState| Msg::ConnectionState(state)
        );

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