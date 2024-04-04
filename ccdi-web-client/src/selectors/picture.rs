use std::sync::Arc;

use base64::{engine::general_purpose::STANDARD, Engine};
// use ccdi_common::RgbImage;
use ccdi_image::{
    // compute_image_stats, render_histogram_as_bmp, rgb_image_to_bmp, ImageStats,
    // Transform,
    TransformFunction,
};

use uuid::Uuid;
use yew::{Callback, Properties};

use wasm_bindgen::JsCast;

use web_sys::console::log_1;
use web_sys::{window, HtmlElement};

use super::*;

// ============================================ PUBLIC =============================================

pub enum Msg {
    ChangeGain(i32),
    ChangeFunction(TransformFunction),
}

pub struct Picture {
    gain: i32,
    function: TransformFunction,
    uuid: String,
}

#[derive(Clone, PartialEq, Properties)]
pub struct PictureData {
    pub image: Option<Arc<Vec<u8>>>,
    pub hist_width: usize,
    pub hist_height: usize,
    pub onresize: Callback<(i32, i32)>,
}

impl Component for Picture {
    type Message = Msg;
    type Properties = PictureData;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            gain: 1,
            function: TransformFunction::Sqrt,
            uuid: Uuid::new_v4().to_string(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ChangeGain(value) => self.gain = value,
            Msg::ChangeFunction(function) => self.function = function,
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        // let transform = Transform {
        //     gain: self.gain,
        //     function: self.function,
        //     sub: 500,
        // };

        let mut width: i32 = 0;
        let mut height: i32 = 0;

        if let Some(window) = window() {
            if let Some(document) = window.document() {
                if let Some(container) =
                    document.get_element_by_id(&format!("imagecanvas-{}", self.uuid))
                {
                    if let Ok(canvas) = container.dyn_into::<HtmlElement>() {
                        log_1(
                            &format!(
                                "container: width: {} height: {}",
                                canvas.offset_width(),
                                canvas.offset_height()
                            )
                            .into(),
                        );
                        width = canvas.offset_width();
                        height = canvas.offset_height();
                        ctx.props().onresize.emit((width, height));
                    }
                }
            }
        }

        html! {
            <div>
                <div class="image-main">
                    <div  class="image-tools">
                        <p>{"View"}</p>
                        <hr />
                        <p>{"Gain"}</p>
                        { gain_button(ctx, self.gain,  1) }
                        { gain_button(ctx, self.gain,  2) }
                        { gain_button(ctx, self.gain,  4) }
                        { gain_button(ctx, self.gain,  8) }
                        { gain_button(ctx, self.gain, 16) }
                        { gain_button(ctx, self.gain, 32) }
                        { gain_button(ctx, self.gain, 64) }
                        <p>{"Func"}</p>
                        { function_button(ctx, self.function, TransformFunction::Linear, "Line") }
                        { function_button(ctx, self.function, TransformFunction::Sqrt, "Sqrt") }
                        { function_button(ctx, self.function, TransformFunction::Log2, "Log2") }
                    </div>
                    <div class="image-content" id={format!("imagecanvas-{}", self.uuid)}>
                        {encoded_image_to_html(ctx.props().image.as_deref())} // TODO: Fix width and height
                        // {histogram_table(stats.as_ref(), hist_h)} // TODO: Fix width of histogram table
                    </div>
                </div>
            </div>
        }
    }
}

fn function_button(
    ctx: &Context<Picture>,
    current_function: TransformFunction,
    button_function: TransformFunction,
    text: &str,
) -> Html {
    let function_click =
        |value: TransformFunction| ctx.link().callback(move |_| Msg::ChangeFunction(value));

    let selected_class = match current_function == button_function {
        true => Some("button-selected"),
        false => None,
    };

    html! {
        <button
            class={classes!("short-button", selected_class)}
            onclick={function_click(button_function)}
        >{ text }</button>
    }
}

fn gain_button(ctx: &Context<Picture>, current_gain: i32, button_gain: i32) -> Html {
    let gain_click = |value: i32| ctx.link().callback(move |_| Msg::ChangeGain(value));

    let selected_class = match current_gain == button_gain {
        true => Some("button-selected"),
        false => None,
    };

    html! {
        <button
            class={classes!("short-button", selected_class)}
            onclick={gain_click(button_gain)}
        >{
            format!("X {}", button_gain)
        }</button>
    }
}

fn encoded_image_to_html(image: Option<&Vec<u8>>) -> Html {
    match image.and_then(png_to_base64) {
        None => html! {},
        Some(ref base64) => html! {
            <img class="contain" src={format!("data:image/bmp;base64,{}", base64)} />
        },
    }
}

// fn histogram_table(stats: Option<&ImageStats>, height: usize) -> Html {
//     match stats {
//         None => html! {},
//         Some(stats) => html! {
//             <div class="hist-table">
//                 <div class="div-table-row">
//                     <div class="hist-table-col">
//                         {limits(stats.total.min, stats.r.min, stats.g.min, stats.b.min)}
//                     </div>
//                     <div class="hist-table-col">
//                         {histogram_image(stats, height)}
//                     </div>
//                     <div class="hist-table-col">
//                         {limits(stats.total.max, stats.r.max, stats.g.max, stats.b.max)}
//                     </div>
//                 </div>
//             </div>
//         },
//     }
// }

// fn histogram_image(stats: &ImageStats, height: usize) -> Html {
//     let payload = render_histogram_as_bmp(stats, height).map(|data| STANDARD.encode(data));

//     match payload {
//         Err(error) => html! { <p>{"Histogram err:"} {error}</p> },
//         Ok(ref base64) => html! {
//             <img class={"gray-border"} src={format!("data:image/bmp;base64,{}", base64)} />
//         },
//     }
// }

fn png_to_base64(image: &Vec<u8>) -> Option<String> {
    let encoded_base64 = STANDARD.encode(image);
    Some(encoded_base64)
}

// fn limits(all: u16, r: u16, g: u16, b: u16) -> Html {
//     html! {
//         <>
//             <div>{all}</div>
//             <hr/>
//             <div class="red">{r}</div>
//             <div class="green">{g}</div>
//             <div class="blue">{b}</div>
//         </>
//     }
// }
