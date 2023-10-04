use wasm_bindgen::{JsCast, UnwrapThrowExt};
use web_sys::{Event, HtmlInputElement, InputEvent};
use yew::prelude::*;

// ============================================ PUBLIC =============================================

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub value: f64,
    pub width: usize,
    pub on_change: Callback<f64>,
}

fn get_value_from_input_event(e: InputEvent, value: f64) -> f64 {
    let event: Event = e.dyn_into().unwrap_throw();
    let event_target = event.target().unwrap_throw();
    let target: HtmlInputElement = event_target.dyn_into().unwrap_throw();
    target.value().parse::<f64>().unwrap_or(value)
}

/// Controlled Text Input Component
#[function_component(FloatInput)]
pub fn usize_input(props: &Props) -> Html {
    let Props { value, width, on_change } = props.clone();

    let oninput = Callback::from(move |input_event: InputEvent| {
        on_change.emit(get_value_from_input_event(input_event, value));
    });

    html! {
        <input type="number" maxlength={width.to_string()} size={width.to_string()} value={value.to_string()} {oninput} />
    }
}