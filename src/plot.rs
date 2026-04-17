use crate::colours::COLOURS;
use crate::colours::Colours;
use crate::expense::Expense;
use crate::processing::day_sums;
use enum_iterator::Sequence;
use image::DynamicImage;
use image::ImageBuffer;
use plotters::chart::ChartBuilder;
use plotters::coord::Shift;
use plotters::prelude::BitMapBackend;
use plotters::prelude::DrawingArea;
use plotters::prelude::IntoDrawingArea;
use plotters::prelude::PathElement;
use plotters::prelude::Pie;
use plotters::series::LineSeries;
use plotters::style::Color as _;
use plotters::style::FontDesc;
use plotters::style::FontFamily;
use plotters::style::FontStyle;
use plotters::style::RGBColor;
use plotters::style::TextStyle;
use plotters::style::text_anchor::HPos;
use plotters::style::text_anchor::Pos;
use plotters::style::text_anchor::VPos;
use std::cmp::min;
use std::fmt::Display;
use std::hash::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher as _;

// TODO: Calculate these dynamically.
const IMAGE_WIDTH: u32 = 1920;
const IMAGE_HEIGHT: u32 = 1080;
const IMAGE_SIZE: (u32, u32) = (IMAGE_WIDTH, IMAGE_HEIGHT);
const FONT_SIZE: f64 = 32.0;
const X_LABEL_AREA_HEIGHT: u32 = 25;
const Y_LABEL_AREA_WIDTH: u32 = 100;
const RELATIVE_PIE_RADIUS: f64 = 0.8;

#[derive(Copy, Clone, Eq, PartialEq, Default, Debug, Sequence)]
pub enum Plot {
    #[default]
    PerDay,
    Pie,
}

impl Plot {
    pub fn title(self) -> &'static str {
        match self {
            Plot::PerDay => "Expenses Per Day",
            Plot::Pie => "Expenses Per Class",
        }
    }

    pub fn make_image<'expenses>(
        self,
        expenses: impl IntoIterator<Item = &'expenses Expense>,
    ) -> anyhow::Result<DynamicImage> {
        match self {
            Plot::PerDay => per_day(expenses),
            Plot::Pie => pie(expenses),
        }
    }
}

impl Display for Plot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Plot::PerDay => write!(f, "Per Day"),
            Plot::Pie => write!(f, "Pie"),
        }
    }
}

fn with_root(
    function: impl FnOnce(
        &DrawingArea<BitMapBackend, Shift>,
        Colours<RGBColor>,
        &TextStyle,
    ) -> anyhow::Result<()>,
) -> anyhow::Result<DynamicImage> {
    let colours = COLOURS.plotters();

    let mut image_buffer = ImageBuffer::new(IMAGE_WIDTH, IMAGE_HEIGHT);

    // TODO: Move all text to the TUI.
    let text_style = TextStyle {
        font: FontDesc::new(FontFamily::SansSerif, FONT_SIZE, FontStyle::Normal),
        color: colours.text.to_backend_color(),
        // Trying to centre the text here makes it, not centred.
        pos: Pos::new(HPos::Left, VPos::Top),
    };

    let root = BitMapBackend::with_buffer(&mut image_buffer, IMAGE_SIZE).into_drawing_area();

    root.fill(&colours.background)?;

    function(&root, colours, &text_style)?;

    root.present()?;

    drop(root);

    Ok(DynamicImage::ImageRgb8(image_buffer))
}

fn per_day<'expenses>(
    expenses: impl IntoIterator<Item = &'expenses Expense>,
) -> anyhow::Result<DynamicImage> {
    let day_sums = day_sums(expenses);

    let start_date = day_sums
        .first_key_value()
        .map(|(day, _)| *day)
        .unwrap_or_default();
    let end_date = day_sums
        .last_key_value()
        .map(|(day, _)| *day)
        .unwrap_or_default();
    let max_expense = day_sums
        .values()
        .copied()
        .max_by(f64::total_cmp)
        .unwrap_or_default();

    with_root(|root, colours, text_style| {
        let mut chart = ChartBuilder::on(root)
            .caption("Expenses Over Time", text_style.clone())
            .margin(10)
            .x_label_area_size(X_LABEL_AREA_HEIGHT)
            .y_label_area_size(Y_LABEL_AREA_WIDTH)
            .build_cartesian_2d(start_date..end_date, 0.0..max_expense)?;

        chart
            .configure_mesh()
            .axis_style(colours.text)
            .bold_line_style(colours.bold_grid)
            .light_line_style(colours.light_grid)
            .label_style(text_style.clone())
            .draw()?;

        chart
            .draw_series(LineSeries::new(day_sums, &colours.graph))?
            .label("Expenses Per Day")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], colours.graph));

        chart
            .configure_series_labels()
            .background_style(colours.background)
            .border_style(colours.border)
            .label_font(text_style.clone())
            .draw()?;

        Ok(())
    })
}

fn pie<'expenses>(
    expenses: impl IntoIterator<Item = &'expenses Expense>,
) -> anyhow::Result<DynamicImage> {
    fn colour(expense: &Expense) -> RGBColor {
        let mut state = DefaultHasher::new();
        expense.class.hash(&mut state);
        let hash = state.finish();
        let [.., red, green, blue] = hash.to_ne_bytes();
        RGBColor(red, green, blue)
    }

    let image_centre = (IMAGE_WIDTH as i32 / 2, IMAGE_HEIGHT as i32 / 2);
    let radius = min(IMAGE_WIDTH, IMAGE_HEIGHT) as f64 / 2.0 * RELATIVE_PIE_RADIUS;

    let mut classes = Vec::<&str>::new();
    let mut amounts = Vec::new();
    let mut colours = Vec::new();

    for expense in expenses {
        if let Some(index) = classes.iter().position(|class| *class == &*expense.class) {
            amounts[index] += expense.amount;
        } else {
            classes.push(&expense.class);
            amounts.push(expense.amount);
            colours.push(colour(expense));
        }
    }

    with_root(|root, _, text_style| {
        let mut pie = Pie::new(&image_centre, &radius, &amounts, &colours, &classes);

        pie.label_style(text_style);
        pie.label_offset(75.0);
        pie.percentages(text_style);

        root.draw(&pie)?;

        Ok(())
    })
}
