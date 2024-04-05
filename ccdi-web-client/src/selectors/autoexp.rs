use autoexp::floatin::FloatInput;
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
}

pub enum Msg {
    UpdatePercentilePix(String),
    UpdatePixelTgt(String),
    UpdatePixelTol(String),
    UpdateMaxExp(String),
    ServerAction(StateMessage),
}

impl Component for AutoExpConfig {
    type Message = Msg;
    type Properties = AutoExpCfgData;

    fn create(ctx: &Context<Self>) -> Self {
        let prop = ctx.props();
        // TODO: Print the values of prop.image_params to check where they are coming from
        Self {
            percentile_pix: prop.image_params.percentile_pix,
            pixel_tgt: prop.image_params.pixel_tgt,
            pixel_tol: prop.image_params.pixel_tol,
            max_exp: prop.image_params.max_exp,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::UpdatePercentilePix(value) => {
                if let Ok(value) = value.parse::<f32>() {
                    if value > 0.0 && value <= 1.0 {
                        self.percentile_pix = value;
                    }
                }
            }
            Msg::UpdatePixelTgt(value) => {
                if let Ok(value) = value.parse::<f32>() {
                    if value > 0.0 {
                        self.pixel_tgt = value;
                    }
                }
            }
            Msg::UpdatePixelTol(value) => {
                if let Ok(value) = value.parse::<f32>() {
                    if value > 0.0 {
                        self.pixel_tol = value;
                    }
                }
            }
            Msg::UpdateMaxExp(value) => {
                if let Ok(value) = value.parse::<f32>() {
                    if value > 0.0 {
                        self.max_exp = value;
                    }
                }
            }
            Msg::ServerAction(action) => ctx.props().on_action.emit(action),
        }
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        use ExposureCommand::*;
        use StateMessage::*;

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
                    <div class="div-table-col w30p"> {"Gain"} </div>
                    <div class="div-table-col w50p">
                    <FloatInput
                        value={format!("{:.3}", self.percentile_pix)}
                        range={(0.0, 1.0)}
                        on_change={chg_percentile_pix}
                    />
                    </div>
                    </div> // div-table-row w100p

                    <div class="div-table-row w100p">
                    <div class="div-table-col w30p"> {"Pixel Target"} </div>
                    <div class="div-table-col w50p">
                    <FloatInput
                        value={format!("{:.5}", self.pixel_tgt)}
                        range={(0.0, 1.0)}
                        on_change={chg_pixel_tgt}
                    />
                    </div>
                    </div> // div-table-row w100p

                    <div class="div-table-row w100p">
                    <div class="div-table-col w30p"> {"Pixel Tolerance"} </div>
                    <div class="div-table-col w50p">
                    <FloatInput
                        value={format!("{:.5}", self.pixel_tol)}
                        range={(0.0, 1.0)}
                        on_change={chg_pixel_tol}
                    />
                    </div>
                    </div> // div-table-row w100p

                    <div class="div-table-row w100p">
                    <div class="div-table-col w30p"> {"Max Exposure"} </div>
                    <div class="div-table-col w50p">
                    <FloatInput
                        value={format!("{:.3}", self.max_exp)}
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
