use argh::FromArgs;

// ============================================ PUBLIC =============================================

#[derive(FromArgs)]
/// CCD imaging service
pub struct ServiceConfig {
    /// run with a demo driver
    #[argh(switch)]
    pub demo: bool,

    /// enable debug logging
    #[argh(switch)]
    pub debug: bool,

    /// log file
    #[argh(option)]
    pub log: Option<String>,
}
