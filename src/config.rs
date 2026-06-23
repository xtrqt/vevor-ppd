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
}

impl Config {
    pub fn parse() -> Self {
        <Self as Parser>::parse()
    }
}
