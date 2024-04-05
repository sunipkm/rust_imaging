use argh::FromArgs;

// ============================================ PUBLIC =============================================

#[derive(FromArgs)]
/// CCD Imaging Service
pub struct ServerConfig {
    /// valid options: "asi", "fli", "demo"
    #[argh(option, default = "String::from(\"asi\")")]
    pub camera: String,

    /// server address
    #[argh(option, default = "default_addr()")]
    pub addr: u16,

    /// enable debug logging
    #[argh(switch)]
    pub debug: bool,

    /// log file
    #[argh(option)]
    pub log: Option<String>,
}

fn default_addr() -> u16 {
    8081
}
