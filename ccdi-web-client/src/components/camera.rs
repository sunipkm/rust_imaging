use std::sync::Arc;

use ccdi_imager_interface::{ImagerProperties, DeviceProperty, BasicProperties};
use yew::Properties;
use super::*;

// ============================================ PUBLIC =============================================

pub struct CameraDetail;

#[derive(Clone, PartialEq, Properties)]
pub struct CameraDetailData {
    pub data: Option<Arc<ImagerProperties>>,
}

impl Component for CameraDetail {
    type Message = ();
    type Properties = CameraDetailData;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        match ctx.props().data.as_ref() {
            Some(properties) => render_properties(properties.as_ref()),
            None => render_missing(),
        }
    }
}

// =========================================== PRIVATE =============================================

fn render_missing() -> Html {
    html!{<div class="div-table">{render_row("No data", "...")}</div>}
}

fn render_properties(properties: &ImagerProperties) -> Html {
    html!{
        <div class="div-table">
            {render_basic_rows(&properties.basic)}
            {render_other_rows(properties)}
        </div>
    }
}

fn render_basic_rows(properties: &BasicProperties) -> Html {
    html!{
        <div>
            {render_row("Resolution", &format!("{} x {}", properties.width, properties.height))}
            {render_row("Temperature", &format!("{}", properties.temperature))}
        </div>
    }
}

fn render_other_rows(properties: &ImagerProperties) -> Html {
    properties.other.iter().map(render_item).collect::<Html>()
}

fn render_item(property: &DeviceProperty) -> Html {
    render_row(&property.name, &property.value)
}

fn render_row(name: &str, value: &str) -> Html {
    html! {
        <div class="div-table-row">
            <div class="div-table-col">{name}</div>
            <div class="div-table-col">{value}</div>
        </div>
    }
}