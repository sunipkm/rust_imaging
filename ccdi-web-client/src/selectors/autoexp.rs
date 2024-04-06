use std::sync::{Arc, Mutex};

use autoexp::floatin::FloatInput;
use once_cell::sync::Lazy;
use web_sys::console::log_1;
use yew::{Callback, Properties};

use super::*;

// ============================================ PUBLIC =============================================

pub struct AutoExpConfig {
    pub percentile_pix: f32,
    pub pixel_tgt: f32,
    pub pixel_tol: f32,
    pub max_exp: f32,
}

#[derive(Clone, PartialEq, Properties)]
pub struct AutoExpCfgData {
    pub on_action: Callback<StateMessage>,
    pub image_params: ImageParams,
    pub view_state: ViewState,
}

pub enum Msg {
    UpdateAll,
    UpdatePercentilePix(f64),
    UpdatePixelTgt(f64),
    UpdatePixelTol(f64),
    UpdateMaxExp(f64),
    ServerAction(StateMessage),
}

impl AutoExpConfig {
    pub fn check(&self, ctx: &Context<Self>) -> bool {
        static LAST_CALL_CAMERA: Lazy<Arc<Mutex<bool>>> = Lazy::new(|| Arc::new(Mutex::new(false)));

        // let update_all = ctx.link().callback(|_| Msg::UpdateAll);

        let cond = ctx.props().view_state.status.camera == ConnectionState::Established;
        let mut last_cond = LAST_CALL_CAMERA.lock().unwrap();

        if cond && !*last_cond {
            ctx.link().send_message(Msg::UpdateAll);
        }

        *last_cond = cond;
        cond
    }
}

impl Component for AutoExpConfig {
    type Message = Msg;
    type Properties = AutoExpCfgData;

    fn create(ctx: &Context<Self>) -> Self {
        let prop = ctx.props();
        Self {
            percentile_pix: prop.image_params.percentile_pix,
            pixel_tgt: prop.image_params.pixel_tgt,
            pixel_tol: prop.image_params.pixel_tol,
            max_exp: prop.image_params.max_exp,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::UpdateAll => {
                log_1(&format!("AutoExpConfig::update: {:?}", ctx.props().image_params).into());
                self.percentile_pix = ctx.props().image_params.percentile_pix;
                self.pixel_tgt = ctx.props().image_params.pixel_tgt;
                self.pixel_tol = ctx.props().image_params.pixel_tol;
                self.max_exp = ctx.props().image_params.max_exp;
            }
            Msg::UpdatePercentilePix(value) => {
                if value > 0.0 && value <= 1.0 {
                    self.percentile_pix = value as f32;
                }
            }
            Msg::UpdatePixelTgt(value) => {
                if value > 0.0 {
                    self.pixel_tgt = value as f32;
                }
            }
            Msg::UpdatePixelTol(value) => {
                if value > 0.0 {
                    self.pixel_tol = value as f32;
                }
            }
            Msg::UpdateMaxExp(value) => {
                if value > 0.0 {
                    self.max_exp = value as f32;
                }
            }
            Msg::ServerAction(action) => ctx.props().on_action.emit(action),
        }
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        use ExposureCommand::*;
        use StateMessage::*;

        self.check(ctx);

        let chg_percentile_pix = ctx.link().callback(Msg::UpdatePercentilePix);
        let chg_pixel_tgt = ctx.link().callback(Msg::UpdatePixelTgt);
        let chg_pixel_tol = ctx.link().callback(Msg::UpdatePixelTol);
        let chg_max_exp = ctx.link().callback(Msg::UpdateMaxExp);

        let server_action = |action: StateMessage| {
            ctx.link()
                .callback(move |_| Msg::ServerAction(action.clone()))
        };

        html! {
        <div>
            <div>
                <p>{"Auto Exposure Configuration"}</p>
                <div class="div-table-row w100p">
                <div class="div-table-col w50p"> {"Percentile Pixel (0--1)"} </div>
                <div class="div-table-col w45p">
                <FloatInput
                    value={self.percentile_pix.to_string()}
                    range={(0.0, 1.0)}
                    sigfig={3}
                    on_change={chg_percentile_pix}
                />
                </div>
                </div> // div-table-row w100p

                <div class="div-table-row w100p">
                <div class="div-table-col w50p"> {"Pixel Target (0--1)"} </div>
                <div class="div-table-col w45p">
                <FloatInput
                    value={self.pixel_tgt.to_string()}
                    sigfig={5}
                    range={(0.0, 1.0)}
                    on_change={chg_pixel_tgt}
                />
                </div>
                </div> // div-table-row w100p

                <div class="div-table-row w100p">
                <div class="div-table-col w50p"> {"Pixel Tolerance (0--1)"} </div>
                <div class="div-table-col w45p">
                <FloatInput
                    value={self.pixel_tol.to_string()}
                    sigfig={5}
                    range={(0.0, 1.0)}
                    on_change={chg_pixel_tol}
                />
                </div>
                </div> // div-table-row w100p

                <div class="div-table-row w100p">
                <div class="div-table-col w50p"> {"Max Exposure (s)"} </div>
                <div class="div-table-col w45p">
                <FloatInput
                    value={self.max_exp.to_string()}
                    sigfig={3}
                    range={(0.001,3600.0)}
                    on_change={chg_max_exp}
                />
                </div>
                </div> // div-table-row w100p

                <button
                    onclick={server_action(ExposureMessage(Update(
                        OptConfigCmd {
                            percentile_pix: self.percentile_pix,
                            pixel_tgt: self.pixel_tgt,
                            pixel_tol: self.pixel_tol,
                            max_exp: self.max_exp,
                        }
                    )))}
                    >{"Update"}</button>
            </div>
        </div>
        }
    }
}
