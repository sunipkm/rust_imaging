use wasm_bindgen::{JsCast, UnwrapThrowExt};
use web_sys::{Event, HtmlInputElement, InputEvent};
use yew::prelude::*;

// ============================================ PUBLIC =============================================

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub value: String,
    pub range: Option<(f64, f64)>,
    pub on_change: Callback<String>,
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
    let Props { value, range, on_change } = props.clone();

    let oninput = Callback::from(move |input_event: InputEvent| {
        on_change.emit(get_value_from_input_event(input_event));
    });

    match range {
        Some((min, max)) => html! {
            <input type="number" min={min.to_string()} max={max.to_string()}
            value={value}
            {oninput} />
        },
        None => html! {
            <input type="number"
            style="width: 100%; padding: 5px; box-sizing:border-box"
            {value} 
            {oninput} />
        }
    }
}