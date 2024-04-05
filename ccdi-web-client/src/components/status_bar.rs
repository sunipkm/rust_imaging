use yew::Properties;
use super::*;

// ============================================ PUBLIC =============================================

/// The `Child` component is the child of the `Parent` component, and will receive updates from the
/// parent using properties.
pub struct StatusBar;

#[derive(Clone, PartialEq, Properties)]
pub struct StatusBarData {
    pub connection: ConnectionState,
    pub logic: LogicStatus,
}

impl Component for StatusBar {
    type Message = ();
    type Properties = StatusBarData;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let main_state = ctx.props().connection;

        html! {
            <div class="status-bar-body float-container">
                { state_view("Connection", main_state) }
                { combined("Camera", main_state, ctx.props().logic.camera) }
                { combined("Storage", main_state, ctx.props().logic.storage.as_connection_state()) }
                { combined("Loop", main_state, ctx.props().logic.loop_enabled) }
                // { combined("Trigger Needed", main_state, ctx.props().logic.required) }
                // { combined("Trigger On", main_state, ctx.props().logic.trigger) }
                { combined("Exposure", main_state, ctx.props().logic.exposure) }
                { combined("Save On", main_state, ctx.props().logic.save) }
            </div>
        }
    }
}

// =========================================== PRIVATE =============================================

fn combined(name: &str, main: ConnectionState, state: ConnectionState) -> Html {
    let status_class = match main {
        ConnectionState::Established => status_to_class(state),
        _other => "unknown"
    };

    state_html(name, status_class)
}

fn state_view(name: &str, state: ConnectionState) -> Html {
    state_html(name, status_to_class(state))
}

fn state_html(name: &str, class: &'static str) -> Html {
    html! {
        <ul class="float-child">
            <li class={classes!("status", class)}>{name}</li>
        </ul>
    }
}

fn status_to_class(state: ConnectionState) -> &'static str {
    match state {
        ConnectionState::Disconnected => "error",
        ConnectionState::Connecting => "warn", //"warn",
        ConnectionState::Established => "ok",
    }
}