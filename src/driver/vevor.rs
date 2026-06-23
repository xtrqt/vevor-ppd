use super::{PrintJob, RasterPage};
use anyhow::{bail, Result};

pub fn render(job: &PrintJob) -> Result<Vec<u8>> {
    let mut out = Vec::new();

    for page in &job.pages {
        render_page(page, job.options.darkness, job.options.speed, &mut out)?;
    }

    Ok(out)
}

fn render_page(page: &RasterPage, darkness: u8, speed: u8, out: &mut Vec<u8>) -> Result<()> {
    if page.width_px == 0 || page.height_px == 0 || page.bytes_per_line == 0 {
        bail!("invalid empty raster page");
    }

    let expected_len = page.bytes_per_line as usize * page.height_px as usize;
    if page.data.len() != expected_len {
        bail!(
            "raster data length {} does not match expected {}",
            page.data.len(),
            expected_len
        );
    }

    // This is the first isolated driver seam. The exact Vevor command dialect
    // still needs to be ported from rastertolabel.c.
    out.extend_from_slice(b"SIZE ");
    out.extend_from_slice(page.width_px.to_string().as_bytes());
    out.extend_from_slice(b" px,");
    out.extend_from_slice(page.height_px.to_string().as_bytes());
    out.extend_from_slice(b" px\n");
    out.extend_from_slice(b"DENSITY ");
    out.extend_from_slice(darkness.to_string().as_bytes());
    out.extend_from_slice(b"\nSPEED ");
    out.extend_from_slice(speed.to_string().as_bytes());
    out.extend_from_slice(b"\nBITMAP ");
    out.extend_from_slice(page.bytes_per_line.to_string().as_bytes());
    out.extend_from_slice(b",0,");
    out.extend_from_slice(page.data.len().to_string().as_bytes());
    out.extend_from_slice(b"\n");
    out.extend_from_slice(&page.data);
    out.extend_from_slice(b"\nPRINT 1\n");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::driver::{LabelOptions, PrintJob, RasterPage};

    #[test]
    fn validates_raster_length() {
        let job = PrintJob {
            pages: vec![RasterPage {
                width_px: 8,
                height_px: 2,
                bytes_per_line: 1,
                data: vec![0xff],
            }],
            options: LabelOptions::default(),
        };

        assert!(render(&job).is_err());
    }

    #[test]
    fn renders_basic_command_stream() {
        let job = PrintJob {
            pages: vec![RasterPage {
                width_px: 8,
                height_px: 2,
                bytes_per_line: 1,
                data: vec![0xff, 0x00],
            }],
            options: LabelOptions::default(),
        };

        let bytes = render(&job).expect("render job");
        assert!(String::from_utf8_lossy(&bytes).contains("PRINT 1"));
    }
}
