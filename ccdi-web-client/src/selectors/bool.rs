use yew::{Properties, Callback};
use super::*;

pub struct BoolSelector;

pub enum Msg {
    SetAutoExp(bool),
}

#[derive(Clone, PartialEq, Properties)]
pub struct SelectorData {
    pub name: &'static str,
    pub value_changed: Callback<bool>,
    pub selected_value: bool,
}

impl Component for BoolSelector {
    type Message = Msg;
    type Properties = SelectorData;

    fn create(_ctx: &Context<Self>) -> Self {
        Self { }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SetAutoExp(value) => {
                ctx.props().value_changed.emit(value)
            },
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let selected = ctx.props().selected_value;

        html! {
            <div>
                <p>{format!("{}: ", &ctx.props().name)}
                {checkbox(selected, ctx)}
                </p>
            </div>
        }
    }
}

fn checkbox(
    current: bool,
    ctx: &Context<BoolSelector>
) -> Html {
    let on_click = |action: bool| ctx.link().callback(move |_| Msg::SetAutoExp(action));

    html! {
        <input type="checkbox" checked={current} onclick={on_click(!current)} />
    }
}
