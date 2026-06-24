use crate::driver::RasterPage;
use futures::io::AsyncReadExt;
use print_raster::reader::{
    cups::unified::CupsRasterUnifiedReader, urf::UrfReader, RasterPageReader, RasterReader,
};
use std::io::Cursor;
use tokio_util::compat::TokioAsyncReadCompatExt;
use tracing::{debug, info};

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
        let bpl = hdr.v1.bytes_per_line;
        let bpc = hdr.v1.bits_per_pixel;
        let cs = hdr.v1.color_space;

        info!(
            width, height, bpl, bpc,
            cs = ?cs,
            "PWG raster page"
        );

        let mut data = Vec::new();
        page.content_mut().read_to_end(&mut data).await?;

        let data_len = data.len();
        let expected = height as usize * bpl as usize;
        info!(data_len, expected, width, height, bpl, "PWG raster data read");

        pages.push(RasterPage {
            width_px: width,
            height_px: height,
            bytes_per_line: bpl,
            data,
        });

        next_page = page.next_page().await?;
    }

    if pages.is_empty() {
        return Err(anyhow::anyhow!("No pages found in raster stream"));
    }

    Ok(pages)
}

pub async fn parse_urf_raster(bytes: &[u8]) -> Result<Vec<RasterPage>, anyhow::Error> {
    let cursor = Cursor::new(bytes);
    let compat_reader = tokio::io::BufReader::new(cursor).compat();
    let reader = UrfReader::new(Box::pin(compat_reader)).await?;

    let mut pages = Vec::new();
    let mut next_page = reader.next_page().await?;

    while let Some(mut page) = next_page {
        let hdr = page.header();
        let width = hdr.width;
        let height = hdr.height;
        let bpp = hdr.bits_per_pixel;
        let cs = hdr.color_space as u8;
        let dpi = hdr.dot_per_inch;
        if bpp == 0 || bpp % 8 != 0 {
            anyhow::bail!("unsupported URF bits_per_pixel: {}", bpp);
        }
        let bpl = width * (bpp as u32 / 8);

        info!(
            width, height, bpp, dpi, bpl,
            cs = ?cs,
            "URF raster page"
        );

        let mut data = Vec::new();
        page.content_mut().read_to_end(&mut data).await?;

        let data_len = data.len();
        let expected = height as usize * bpl as usize;
        info!(data_len, expected, width, height, bpl, "URF raster data read");

        // First 20 bytes of pixel data for debugging
        if data_len > 0 {
            let preview_len = data.len().min(20);
            debug!(preview = ?&data[..preview_len], "URF pixel data preview");
        }

        // Check if row data is all-255 or all-0 per row
        let all_same = data.chunks(width as usize).enumerate().take(5).map(|(i, row)| {
            let first = row[0];
            let uniform = row.iter().all(|&b| b == first);
            (i, first, uniform)
        }).collect::<Vec<_>>();
        debug!(?all_same, "URF first 5 rows sample");

        pages.push(RasterPage {
            width_px: width,
            height_px: height,
            bytes_per_line: bpl,
            data,
        });

        next_page = page.next_page().await?;
    }

    if pages.is_empty() {
        return Err(anyhow::anyhow!("No pages found in raster stream"));
    }

    info!(page_count = pages.len(), "URF raster parsed");

    Ok(pages)
}
