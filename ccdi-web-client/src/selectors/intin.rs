use wasm_bindgen::{JsCast, UnwrapThrowExt};
use web_sys::{Event, HtmlInputElement, InputEvent};
use yew::prelude::*;

// ============================================ PUBLIC =============================================

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub value: i64,
    pub width: usize,
    pub on_change: Callback<String>,
}

fn get_value_from_input_event(e: InputEvent) -> String {
    let event: Event = e.dyn_into().unwrap_throw();
    let event_target = event.target().unwrap_throw();
    let target: HtmlInputElement = event_target.dyn_into().unwrap_throw();
    target.value()
}

/// Controlled Text Input Component
#[function_component(IntInput)]
pub fn text_input(props: &Props) -> Html {
    let Props { value, width, on_change } = props.clone();

    let oninput = Callback::from(move |input_event: InputEvent| {
        on_change.emit(get_value_from_input_event(input_event));
    });

    let value = value.to_string();

    html! {
        <input type="number" maxwidth={width.to_string()} size={width.to_string()} {value} {oninput} />
    }
}