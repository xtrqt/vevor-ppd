use crate::driver::RasterPage;
use futures::io::AsyncReadExt;
use print_raster::reader::{
    cups::unified::CupsRasterUnifiedReader, RasterPageReader, RasterReader,
};
use std::io::Cursor;
use tokio_util::compat::TokioAsyncReadCompatExt;

pub async fn parse_pwg_raster(bytes: &[u8]) -> Result<Vec<RasterPage>, anyhow::Error> {
    let cursor = Cursor::new(bytes);
    let compat_reader = tokio::io::BufReader::new(cursor).compat();
    let reader = CupsRasterUnifiedReader::new(Box::pin(compat_reader)).await?;

    let mut pages = Vec::new();
    let mut next_page = reader.next_page().await?;

    while let Some(mut page) = next_page {
        let hdr = page.header();
        let width = hdr.v1.width;
        let height = hdr.v1.height;
        let bytes_per_line = hdr.v1.bytes_per_line;

        let mut data = Vec::new();
        page.content_mut().read_to_end(&mut data).await?;

        pages.push(RasterPage {
            width_px: width,
            height_px: height,
            bytes_per_line,
            data,
        });

        next_page = page.next_page().await?;
    }

    if pages.is_empty() {
        return Err(anyhow::anyhow!("No pages found in raster stream"));
    }

    Ok(pages)
}
