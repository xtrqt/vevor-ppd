pub mod vevor;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LabelOptions {
    pub dpi: u16,
    pub label_width_mm: u32,
    pub label_height_mm: u32,
    pub darkness: u8,
    pub speed: u8,
    pub gap_mm: u8,
    pub gap_offset_mm: u8,
    pub feed_offset_mm: i8,
    pub rotate: u16,
}

impl Default for LabelOptions {
    fn default() -> Self {
        Self {
            dpi: 300,
            label_width_mm: 40,
            label_height_mm: 30,
            darkness: 8,
            speed: 4,
            gap_mm: 3,
            gap_offset_mm: 0,
            feed_offset_mm: 0,
            rotate: 0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RasterPage {
    pub width_px: u32,
    pub height_px: u32,
    pub bytes_per_line: u32,
    pub dot_per_inch: u32,
    pub data: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrintJob {
    pub pages: Vec<RasterPage>,
    pub options: LabelOptions,
}
