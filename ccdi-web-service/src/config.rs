use argh::FromArgs;

// ============================================ PUBLIC =============================================

#[derive(FromArgs)]
/// CCD imaging service
pub struct ServiceConfig {
    /// run with a demo driver
    #[argh(switch, short = 'm')]
    pub demo: bool,
}