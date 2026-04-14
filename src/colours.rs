use anyhow::Context;
use dark_light::Mode;
use plotters::style::RGBColor;

pub struct Colours {
    pub text: RGBColor,
    pub background: RGBColor,
    pub border: RGBColor,
    pub bold_grid: RGBColor,
    pub light_grid: RGBColor,
    pub graph: RGBColor,
}

pub fn colours() -> anyhow::Result<Colours> {
    let mode = dark_light::detect().context("detetcting the system colour scheme")?;

    Ok(match mode {
        Mode::Dark => Colours {
            text: RGBColor(0xe0, 0xde, 0xf4),
            background: RGBColor(0x19, 0x17, 0x24),
            border: RGBColor(0x52, 0x4f, 0x67),
            bold_grid: RGBColor(0x40, 0x3d, 0x52),
            light_grid: RGBColor(0x21, 0x20, 0x2e),
            graph: RGBColor(0xeb, 0x6f, 0x92),
        },
        Mode::Light | Mode::Unspecified => Colours {
            text: RGBColor(0x57, 0x52, 0x79),
            background: RGBColor(0xfa, 0xf4, 0xed),
            border: RGBColor(0xce, 0xca, 0xcd),
            bold_grid: RGBColor(0xdf, 0xda, 0xd9),
            light_grid: RGBColor(0xf4, 0xed, 0xe8),
            graph: RGBColor(0xb4, 0x63, 0x7a),
        },
    })
}
