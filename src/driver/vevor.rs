use super::{LabelOptions, PrintJob, RasterPage};
use anyhow::{bail, Result};
use tracing::info;

pub fn render(job: &PrintJob) -> Result<Vec<u8>> {
    let mut out = Vec::new();

    for (i, page) in job.pages.iter().enumerate() {
        info!(
            page = i,
            width_px = page.width_px,
            height_px = page.height_px,
            bytes_per_line = page.bytes_per_line,
            data_len = page.data.len(),
            "rendering page"
        );
        render_page(page, &job.options, &mut out)?;
    }

    info!(total_output_bytes = out.len(), "render complete");
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

    let bitmap_bytes_per_line = page.width_px.div_ceil(8);

    info!(
        width_px = page.width_px,
        height_px = page.height_px,
        bitmap_bytes_per_line,
        label_width_mm = options.label_width_mm,
        label_height_mm = options.label_height_mm,
        "computed label geometry"
    );

    if options.label_width_mm == 0 || options.label_height_mm == 0 {
        bail!(
            "zero-size label: {}x{}mm",
            options.label_width_mm,
            options.label_height_mm
        );
    }

    append_line(
        out,
        format_args!(
            "SIZE {} mm,{} mm",
            options.label_width_mm, options.label_height_mm
        ),
    );
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
                dot_per_inch: 300,
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
                dot_per_inch: 300,
                data: vec![0, 255, 0, 255, 0, 255, 0, 255],
            }],
            options: LabelOptions::default(),
        };

        let bytes = render(&job).expect("render job");
        let text = String::from_utf8_lossy(&bytes);
        assert!(text.contains("SIZE 40 mm,30 mm"));
        assert!(text.contains("BITMAP 0,0,1,1,1,"));
        assert!(text.contains("PRINT 1,1"));
        // Verify the packed pixel byte for alternating black/white
        // input: [0,255,0,255,0,255,0,255]
        // mask sequence: 0x80,0x40,0x20,0x10,0x08,0x04,0x02,0x01
        // pixels ≤200 set bit → 0x80|0x20|0x08|0x02 = 0xAA
        // then !0xAA = 0x55
        // BITMAP 0,0,1,1,1,  is 17 chars, data starts at byte 17 after the 'B'
        let bitm = text.find("BITMAP 0,0,1,1,1,").unwrap();
        let data_start = bitm + 17;
        assert_eq!(bytes[data_start], 0x55, "first bitmap byte mismatch");
    }

    /// Build expected TSPL output using the exact C reference logic for BEEPRT.
    /// Used as a fixture to compare against render().
    fn c_reference_render(width_px: u32, height_px: u32, bpl: u32, data: &[u8]) -> Vec<u8> {
        let bitmap_bpl = (width_px + 7) / 8;

        let mut out = Vec::new();

        // StartPage BEEPRT (lines 373-456)
        out.extend_from_slice(b"SIZE 40 mm,30 mm\r\n");
        out.extend_from_slice(b"REFERENCE 0,0\r\n");
        out.extend_from_slice(b"DIRECTION 0,0\r\n");
        out.extend_from_slice(b"GAP 3 mm,0 mm\r\n");
        out.extend_from_slice(b"OFFSET 0 mm\r\n");
        out.extend_from_slice(b"DENSITY 8\r\n");
        out.extend_from_slice(b"SPEED 4\r\n");
        out.extend_from_slice(b"SETC AUTODOTTED OFF\r\n");
        out.extend_from_slice(b"SETC PAUSEKEY ON\r\n");
        out.extend_from_slice(b"SETC WATERMARK OFF\r\n");
        out.extend_from_slice(b"CLS\r\n");
        out.extend_from_slice(b"BITMAP 0,0,");
        out.extend_from_slice(bitmap_bpl.to_string().as_bytes());
        out.extend_from_slice(b",");
        out.extend_from_slice(height_px.to_string().as_bytes());
        out.extend_from_slice(b",1,");

        // OutputLine BEEPRT (lines 1040-1053)
        for y in 0..height_px as usize {
            let row_start = y * bpl as usize;
            let row = &data[row_start..row_start + bpl as usize];
            let mut i = 0usize;
            while i < bpl as usize {
                let mut packed = 0u8;
                let mut mask = 0x80u8;
                while mask != 0 && i < bpl as usize {
                    if row[i] <= 200 {
                        packed |= mask;
                    }
                    i += 1;
                    mask >>= 1;
                }
                out.push(!packed);
            }
        }

        // EndPage BEEPRT (line 842)
        out.extend_from_slice(b"\nPRINT 1,1\r\n");

        out
    }

    #[test]
    fn pixel_conversion_matches_c_reference() {
        // Simulate a 40x30mm label at 300 DPI:
        // width = ceil(40 * 300 / 25.4) ≈ 473 px
        // height = ceil(30 * 300 / 25.4) ≈ 354 px
        // 8-bit grayscale: bytes_per_line = width
        let w = 12u32;
        let h = 12u32;

        // Create a checkerboard pattern: 2x2 pixel black/white tiles
        let mut data = Vec::with_capacity((w * h) as usize);
        for y in 0..h {
            for x in 0..w {
                // Even tile = black (0), odd tile = white (255)
                if ((x / 2) + (y / 2)) % 2 == 0 {
                    data.push(0u8); // black
                } else {
                    data.push(255u8); // white
                }
            }
        }

        let dot_per_inch = 300u32;
        let job = PrintJob {
            pages: vec![RasterPage {
                width_px: w,
                height_px: h,
                bytes_per_line: w,
                dot_per_inch,
                data: data.clone(),
            }],
            options: LabelOptions::default(),
        };

        let rust_output = render(&job).expect("render");
        let c_output = c_reference_render(w, h, w, &data);

        // Compare command headers portion (up to BITMAP data)
        let cmd_end = rust_output
            .windows(5)
            .position(|w| w == b"1,\nPR")
            .unwrap_or(rust_output.len().saturating_sub(10));
        let c_cmd_end = c_output
            .windows(5)
            .position(|w| w == b"1,\nPR")
            .unwrap_or(c_output.len().saturating_sub(10));

        assert_eq!(
            &rust_output[..cmd_end],
            &c_output[..c_cmd_end],
            "command headers differ"
        );

        // Compare full output
        assert_eq!(
            rust_output, c_output,
            "full output differs from C reference"
        );
    }

    #[test]
    fn bitmap_byte_count_and_data_match() {
        let w = 40u32;
        let h = 30u32;
        // All medium-gray pixels (will be thresholded)
        let data = vec![128u8; (w * h) as usize];

        let job = PrintJob {
            pages: vec![RasterPage {
                width_px: w,
                height_px: h,
                bytes_per_line: w,
                dot_per_inch: 300,
                data,
            }],
            options: LabelOptions::default(),
        };

        let bytes = render(&job).expect("render");
        let text = String::from_utf8_lossy(&bytes);

        // 40px width / 8 = 5 bytes per bitmap row, 30 rows = 150 bytes
        let bitmap_bpl = w.div_ceil(8);
        assert!(
            text.contains(&format!("BITMAP 0,0,{bitmap_bpl},{h},1,")),
            "BITMAP command mismatch"
        );

        // Gray (128) ≤ 200 so pixels should be set as black
        // packed = 0xFF for each 8-pixel chunk, then !packed = 0x00
        let bitmap_marker = format!(",{h},1,");
        let bitmap_start = bytes
            .windows(bitmap_marker.len())
            .position(|w| w == bitmap_marker.as_bytes())
            .map(|pos| pos + bitmap_marker.len())
            .unwrap();
        let bitmap_end = bytes.len() - b"\nPRINT 1,1\r\n".len();
        let bitmap_data = &bytes[bitmap_start..bitmap_end];

        assert_eq!(
            bitmap_data.len(),
            (bitmap_bpl * h) as usize,
            "bitmap data length mismatch"
        );
        assert!(
            bitmap_data.iter().all(|&b| b == 0x00),
            "all pixels ≤200 should produce 0x00 bytes"
        );
    }
}
