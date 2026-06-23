use clap::Parser;
use std::net::SocketAddr;
use std::path::PathBuf;

#[derive(Clone, Debug, Parser)]
#[command(author, version, about)]
pub struct Config {
    /// Address used by the HTTP/IPP server.
    #[arg(long, env = "LISTEN_ADDR", default_value = "0.0.0.0:631")]
    pub listen_addr: SocketAddr,

    /// Output device path. Use a temporary file for dry runs.
    #[arg(long, env = "OUTPUT_DEVICE", default_value = "/dev/usb/lp0")]
    pub output_device: PathBuf,

    /// Human-readable printer name advertised over IPP.
    #[arg(long, env = "PRINTER_NAME", default_value = "Vevor Label Printer 300")]
    pub printer_name: String,

    /// Public printer URI advertised to clients.
    #[arg(
        long,
        env = "PRINTER_URI",
        default_value = "ipp://localhost:631/ipp/print"
    )]
    pub printer_uri: String,

    /// Enable Bonjour/mDNS advertisement for macOS/iOS discovery.
    #[arg(long, env = "ENABLE_BONJOUR", default_value_t = false)]
    pub enable_bonjour: bool,

    /// Bonjour service instance name.
    #[arg(
        long,
        env = "BONJOUR_SERVICE_NAME",
        default_value = "Vevor Label Printer 300"
    )]
    pub bonjour_service_name: String,

    /// Host address advertised in admin URLs and IPP metadata.
    #[arg(long, env = "PRINTER_HOST", default_value = "localhost")]
    pub printer_host: String,

    /// Stable UUID advertised to Bonjour clients.
    #[arg(
        long,
        env = "BONJOUR_UUID",
        default_value = "8a8a9a2d-43dc-4c7f-8fd3-0e4f03000001"
    )]
    pub bonjour_uuid: String,
}

impl Config {
    pub fn parse() -> Self {
        <Self as Parser>::parse()
    }
}
