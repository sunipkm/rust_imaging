use ccdi_common::ExposureCommand;
use web_sys::console::log_1;
use yew::{Properties, Callback};
use super::*;

// ============================================ PUBLIC =============================================

pub struct CompositionDetail;

#[derive(Clone, PartialEq, Properties)]
pub struct CompositionDetailData {
    pub on_action: Callback<StateMessage>,
    pub image_params: ImageParams,
    pub camera_params: CameraParams,
}

pub enum Msg{
    ServerAction(StateMessage)
}

impl Component for CompositionDetail {
    type Message = Msg;
    type Properties = CompositionDetailData;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }


    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ServerAction(action) => ctx.props().on_action.emit(action),
        }
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        use StateMessage::*;
        use CameraParamMessage::*;

        let loop_enabled = ctx.props().camera_params.loop_enabled;
        // let trigger_required = ctx.props().camera_params.trigger_required;

        let server_action = |action: StateMessage| ctx.link().callback(
            move |_| {
                log_1(&format!("Sending action: {:?}", action).into());
                Msg::ServerAction(action.clone()) }
        );

        html!{
            <div>
                <div>
                    <p>{"Exposition"}</p>
                    <button
                        class={classes!(if loop_enabled { Some("button-selected") } else { None })}
                        onclick={server_action(CameraParam(EnableLoop(true)))}
                        >{"Loop ON"}
                    </button>
                    <button
                        class={classes!(if !loop_enabled { Some("button-selected") } else { None })}
                        onclick={server_action(CameraParam(EnableLoop(false)))}
                        >{"Loop OFF"}
                    </button>
                    <button
                        class={classes!(if loop_enabled { Some("button-disabled") } else { None })}
                        onclick={
                        server_action(ExposureMessage(ExposureCommand::Start))
                    }>{"Expose"}</button>
                </div>
                // <div>
                //     <button
                //         class={classes!(if !trigger_required { Some("button-selected") } else { None })}
                //         onclick={server_action(CameraParam(SetTriggerRequired(false)))}
                //         >{"Trigger OFF"}
                //     </button>
                //     <button
                //         class={classes!(if trigger_required { Some("button-selected") } else { None })}
                //         onclick={server_action(CameraParam(SetTriggerRequired(true)))}
                //         >{"Trigger ON"}
                //     </button>
                // </div>
            </div>
        }
    }
}
