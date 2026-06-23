use super::{LabelOptions, PrintJob, RasterPage};
use anyhow::{bail, Result};

pub fn render(job: &PrintJob) -> Result<Vec<u8>> {
    let mut out = Vec::new();

    for page in &job.pages {
        render_page(page, &job.options, &mut out)?;
    }

    Ok(out)
}

fn render_page(page: &RasterPage, options: &LabelOptions, out: &mut Vec<u8>) -> Result<()> {
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

    let dots_per_mm = dots_per_mm(options.dpi);
    let width_mm = div_round_up(page.width_px, dots_per_mm);
    let height_mm = div_round_up(page.height_px, dots_per_mm);
    let bitmap_bytes_per_line = page.width_px.div_ceil(8);

    append_line(out, format_args!("SIZE {width_mm} mm,{height_mm} mm"));
    append_line(out, format_args!("REFERENCE 0,0"));
    append_line(out, format_args!("DIRECTION {},0", options.rotate));
    append_line(
        out,
        format_args!("GAP {} mm,{} mm", options.gap_mm, options.gap_offset_mm),
    );
    append_line(out, format_args!("OFFSET {} mm", options.feed_offset_mm));
    append_line(out, format_args!("DENSITY {}", options.darkness));
    append_line(out, format_args!("SPEED {}", options.speed));
    append_line(out, format_args!("SETC AUTODOTTED OFF"));
    append_line(out, format_args!("SETC PAUSEKEY ON"));
    append_line(out, format_args!("SETC WATERMARK OFF"));
    append_line(out, format_args!("CLS"));
    out.extend_from_slice(
        format!("BITMAP 0,0,{},{},1,", bitmap_bytes_per_line, page.height_px).as_bytes(),
    );

    for y in 0..page.height_px as usize {
        let start = y * page.bytes_per_line as usize;
        let end = start + page.bytes_per_line as usize;
        write_bitmap_row(
            &page.data[start..end],
            page.width_px,
            page.bytes_per_line,
            bitmap_bytes_per_line,
            out,
        )?;
    }

    out.extend_from_slice(b"\nPRINT 1,1\r\n");

    Ok(())
}

fn write_bitmap_row(
    row: &[u8],
    width_px: u32,
    bytes_per_line: u32,
    bitmap_bytes_per_line: u32,
    out: &mut Vec<u8>,
) -> Result<()> {
    if bytes_per_line == width_px {
        for chunk in row.chunks(8) {
            let mut packed = 0u8;
            let mut mask = 0x80u8;
            for pixel in chunk {
                if *pixel <= 200 {
                    packed |= mask;
                }
                mask >>= 1;
            }
            out.push(!packed);
        }
        return Ok(());
    }

    if bytes_per_line == bitmap_bytes_per_line {
        out.extend(row.iter().map(|byte| !byte));
        return Ok(());
    }

    bail!(
        "unsupported raster row width: got {} bytes, expected {} grayscale or {} packed bytes",
        bytes_per_line,
        width_px,
        bitmap_bytes_per_line
    );
}

fn dots_per_mm(dpi: u16) -> u32 {
    div_round_up(10 * dpi as u32, 254)
}

fn div_round_up(value: u32, divisor: u32) -> u32 {
    value.div_ceil(divisor)
}

fn append_line(out: &mut Vec<u8>, args: std::fmt::Arguments<'_>) {
    use std::fmt::Write;

    let mut line = String::new();
    let _ = line.write_fmt(args);
    line.push_str("\r\n");
    out.extend_from_slice(line.as_bytes());
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
                height_px: 1,
                bytes_per_line: 8,
                data: vec![0, 255, 0, 255, 0, 255, 0, 255],
            }],
            options: LabelOptions::default(),
        };

        let bytes = render(&job).expect("render job");
        let text = String::from_utf8_lossy(&bytes);
        assert!(text.contains("SIZE 1 mm,1 mm"));
        assert!(text.contains("BITMAP 0,0,1,1,1,"));
        assert!(text.contains("PRINT 1,1"));
    }
}
