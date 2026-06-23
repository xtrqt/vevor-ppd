use anyhow::{Context, Result};
use std::path::Path;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;

pub async fn write_all(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .await
        .with_context(|| format!("failed to open output device {}", path.display()))?;

    file.write_all(bytes)
        .await
        .with_context(|| format!("failed to write output device {}", path.display()))?;
    file.flush().await?;

    Ok(())
}
