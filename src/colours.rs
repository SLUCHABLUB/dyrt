use anyhow::Context;
use dark_light::Mode;
use image::Pixel;
use image::Rgb;
use image::Rgba;
use plotters::style::RGBColor;

#[derive(Copy, Clone)]
pub struct Colours<Colour = Rgb<u8>> {
    pub text: Colour,
    pub background: Colour,
    pub border: Colour,
    pub bold_grid: Colour,
    pub light_grid: Colour,
    pub graph: Colour,
}

impl Colours {
    pub fn plotters(self) -> Colours<RGBColor> {
        self.map(|Rgb([red, green, blue])| RGBColor(red, green, blue))
    }

    pub fn rgba(self) -> Colours<Rgba<u8>> {
        self.map(|colour| colour.to_rgba())
    }
}

impl<Colour> Colours<Colour> {
    fn map<F, NewColour>(self, mut function: F) -> Colours<NewColour>
    where
        F: FnMut(Colour) -> NewColour,
    {
        Colours {
            text: function(self.text),
            background: function(self.background),
            border: function(self.border),
            bold_grid: function(self.bold_grid),
            light_grid: function(self.light_grid),
            graph: function(self.graph),
        }
    }
}

pub fn detect_colours() -> anyhow::Result<Colours> {
    let mode = dark_light::detect().context("detecting the system colour scheme")?;

    Ok(match mode {
        Mode::Dark => Colours {
            text: Rgb([0xe0, 0xde, 0xf4]),
            background: Rgb([0x19, 0x17, 0x24]),
            border: Rgb([0x52, 0x4f, 0x67]),
            bold_grid: Rgb([0x40, 0x3d, 0x52]),
            light_grid: Rgb([0x21, 0x20, 0x2e]),
            graph: Rgb([0xeb, 0x6f, 0x92]),
        },
        Mode::Light | Mode::Unspecified => Colours {
            text: Rgb([0x57, 0x52, 0x79]),
            background: Rgb([0xfa, 0xf4, 0xed]),
            border: Rgb([0xce, 0xca, 0xcd]),
            bold_grid: Rgb([0xdf, 0xda, 0xd9]),
            light_grid: Rgb([0xf4, 0xed, 0xe8]),
            graph: Rgb([0xb4, 0x63, 0x7a]),
        },
    })
}
