use wasm_bindgen::{JsCast, UnwrapThrowExt};
use web_sys::{Event, HtmlInputElement, InputEvent};
use yew::prelude::*;

// ============================================ PUBLIC =============================================

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub value: usize,
    pub on_change: Callback<usize>,
}

fn get_value_from_input_event(e: InputEvent, value: usize) -> usize {
    let event: Event = e.dyn_into().unwrap_throw();
    let event_target = event.target().unwrap_throw();
    let target: HtmlInputElement = event_target.dyn_into().unwrap_throw();
    target.value().parse::<usize>().unwrap_or(value)
}

/// Controlled Text Input Component
#[function_component(UsizeInput)]
pub fn usize_input(props: &Props) -> Html {
    let Props { value, on_change } = props.clone();

    let oninput = Callback::from(move |input_event: InputEvent| {
        on_change.emit(get_value_from_input_event(input_event, value));
    });

    html! {
        <input 
        type="number" 
        style="width: 95%; padding: 5px; box-sizing:border-box" 
        value={value.to_string()} 
        {oninput} />
    }
}
