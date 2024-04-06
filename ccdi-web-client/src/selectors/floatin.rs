#![allow(unused_variables)]
use gloo::console;
use wasm_bindgen::{JsCast, UnwrapThrowExt};
use web_sys::{Event, HtmlInputElement, InputEvent};
use yew::prelude::*;

// ============================================ PUBLIC =============================================

// pub struct FloatInput {
//     pub strvalue: String,
//     pub lastvalue: String,
//     pub onchange: Callback<InputEvent>,
// }

// pub enum Msg {
//     SetValue(String),
// }

// #[derive(Clone, PartialEq, Properties)]
// pub struct FloatInputData {
//     pub on_change: Callback<f64>,
//     pub value: f64,
//     pub sigfig: usize,
//     pub range: Option<(f64, f64)>,
//     pub update: Option<Callback<(), bool>>,
// }

// impl Component for FloatInput {
//     type Message = Msg;
//     type Properties = FloatInputData;

//     fn create(ctx: &Context<Self>) -> Self {
//         let props = ctx.props();
//         Self {
//             strvalue: format!("{:.1$}", props.value, props.sigfig),
//             lastvalue: format!("{:.1$}", props.value, props.sigfig),
//             onchange: ctx.link().callback(move |evt: InputEvent| {
//                 let value = get_value_from_input_event(evt);
//                 console::info!("Setting value:", value.clone());
//                 Msg::SetValue(value)
//             }),
//         }
//     }

//     fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
//         match msg {
//             Msg::SetValue(value) => {
//                 console::info!("onchange triggered:", value.clone());
//                 if let Ok(value) = value.parse::<f64>() {
//                     self.strvalue = format!("{:.1$}", value, ctx.props().sigfig);
//                     self.lastvalue = self.strvalue.clone();
//                     ctx.props().on_change.emit(value);
//                 } else {
//                     console::info!("Invalid value:", value.clone());
//                     self.strvalue = self.lastvalue.clone();
//                 }
//                 self.strvalue = value;
//             }
//         }
//         true
//     }

//     fn view(&self, ctx: &Context<Self>) -> Html {
//         console::info!("Rendering FloatInput:", self.strvalue.clone());
//         let props = ctx.props();
//         // let on_change_p1 = ctx.link().callback(Msg::SetValue);

//         match props.range {
//             Some((min, max)) => html! {
//                 <input type="number" min={min.to_string()} max={max.to_string()}
//                 style="width: 100%; padding: 5px; box-sizing:border-box"
//                 value={self.strvalue.clone()}
//                 oninput={self.onchange.clone()}
//                 />
//             },
//             None => html! {
//                 <input type="number"
//                 style="width: 100%; padding: 5px; box-sizing:border-box"
//                 value={self.strvalue.clone()}
//                 oninput={self.onchange.clone()}
//                 />
//             },
//         }
//     }
// }

// fn get_value_from_input_event(e: InputEvent) -> String {
//     let event: Event = e.dyn_into().unwrap_throw();
//     let event_target = event.target().unwrap_throw();
//     let target: HtmlInputElement = event_target.dyn_into().unwrap_throw();
//     target.value()
// }

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub value: String,
    pub sigfig: usize,
    pub range: Option<(f64, f64)>,
    pub on_change: Callback<f64>,
}

fn get_value_from_input_event(e: InputEvent) -> String {
    let event: Event = e.dyn_into().unwrap_throw();
    let event_target = event.target().unwrap_throw();
    let target: HtmlInputElement = event_target.dyn_into().unwrap_throw();
    target.value()
}

/// Controlled Text Input Component
#[function_component(FloatInput)]
pub fn text_input(props: &Props) -> Html {
    let Props {
        value,
        sigfig,
        range,
        on_change,
    } = props.clone();

    let oninput = Callback::from(move |input_event: InputEvent| {
        let value = get_value_from_input_event(input_event);
        console::info!("Setting value:", value.clone());
        if let Ok(value) = value.parse::<f64>() {
            on_change.emit(value);
        }
    });

    match range {
        Some((min, max)) => html! {
            <input type="number" min={min.to_string()} max={max.to_string()}
            style="width: 100%; padding: 5px; box-sizing:border-box"
            {value}
            {oninput} />
        },
        None => html! {
            <input type="number"
            style="width: 100%; padding: 5px; box-sizing:border-box"
            {value}
            {oninput} />
        },
    }
}
