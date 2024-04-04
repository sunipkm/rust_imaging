use super::*;
use crate::selectors::rawhtml::SafeHtml;
use yew::{html_nested, Properties};

// ============================================ PUBLIC =============================================

/// The `Child` component is the child of the `Parent` component, and will receive updates from the
/// parent using properties.
pub struct ShootingDetails;

#[derive(Clone, PartialEq, Properties)]
pub struct ShootingDetailsData {
    pub storage_details: StorageDetail,
}

impl Component for ShootingDetails {
    type Message = ();
    type Properties = ShootingDetailsData;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let details = &ctx.props().storage_details;

        html! {
            <div class="div-table w100p">
                {render_detail_rows(details)}
            </div>
        }
    }
}

// =========================================== PRIVATE =============================================

fn render_detail_rows(properties: &StorageDetail) -> Html {
    properties
        .storage_log
        .iter()
        .map(render_item)
        .collect::<Html>()
}

fn render_item(property: &StorageLogRecord) -> Html {
    let name = property.name.rsplit('/').next().unwrap_or_default();
    html! {
        <div class="div-table-row w100p">
        <div class="div-table-col w5p">
        <SafeHtml html={
            match &property.status {
                StorageLogStatus::Success => "✅".to_owned(),
                StorageLogStatus::Error(error) => {
                    html_nested! {format!("<div class='tooltip'> ❌
                    <span class='tooltiptext'>{error}</span>
                    </div>")}
                },
            }
        }/>
        </div>
        <div class="div-table-col w90p">{name}</div>
        </div>
    }
}
