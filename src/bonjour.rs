use crate::config::Config;
use anyhow::{Context, Result};
use mdns_sd::{ServiceDaemon, ServiceInfo};
use tracing::info;

const SERVICE_TYPE: &str = "_ipp._tcp.local.";

pub struct Advertiser {
    daemon: Option<ServiceDaemon>,
}

impl Advertiser {
    pub fn start(config: &Config) -> Result<Self> {
        if !config.enable_bonjour {
            return Ok(Self { daemon: None });
        }

        let daemon = ServiceDaemon::new().context("failed to start mDNS daemon")?;
        let host_name = format!("{}.local.", hostname_slug(&config.bonjour_service_name));
        let admin_url = format!(
            "http://{}:{}/",
            config.printer_host,
            config.listen_addr.port()
        );
        let properties = [
            ("txtvers", "1"),
            ("qtotal", "1"),
            ("rp", "ipp/print"),
            ("ty", config.printer_name.as_str()),
            ("product", "(Vevor Label Printer 300)"),
            ("pdl", "image/pwg-raster"),
            ("adminurl", admin_url.as_str()),
            ("UUID", config.bonjour_uuid.as_str()),
            ("Color", "F"),
            ("Duplex", "F"),
            ("note", "Vevor Label Printer"),
            ("priority", "50"),
        ];

        let service = ServiceInfo::new(
            SERVICE_TYPE,
            &config.bonjour_service_name,
            &host_name,
            &config.printer_host,
            config.listen_addr.port(),
            &properties[..],
        )
        .context("failed to create Bonjour service info")?;

        daemon
            .register(service)
            .context("failed to register Bonjour IPP service")?;
        info!(service = %config.bonjour_service_name, "registered Bonjour IPP service");

        Ok(Self {
            daemon: Some(daemon),
        })
    }
}

impl Drop for Advertiser {
    fn drop(&mut self) {
        if let Some(daemon) = self.daemon.take() {
            let _ = daemon.shutdown();
        }
    }
}

fn hostname_slug(value: &str) -> String {
    let slug: String = value
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();

    slug.trim_matches('-').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_dns_safe_hostname_slug() {
        assert_eq!(
            hostname_slug("Vevor Label Printer 300"),
            "vevor-label-printer-300"
        );
    }
}
