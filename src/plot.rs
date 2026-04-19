use crate::colours::Colours;
use crate::expense::Expense;
use crate::processing::day_sums;
use enum_iterator::Sequence;
use image::DynamicImage;
use image::ImageBuffer;
use plotters::chart::ChartBuilder;
use plotters::chart::ChartContext;
use plotters::coord::Shift;
use plotters::coord::ranged1d::AsRangedCoord;
use plotters::coord::ranged1d::ValueFormatter;
use plotters::prelude::BitMapBackend;
use plotters::prelude::Cartesian2d;
use plotters::prelude::DrawingArea;
use plotters::prelude::IntoDrawingArea;
use plotters::prelude::PathElement;
use plotters::prelude::Pie;
use plotters::prelude::Ranged;
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
const FONT_SIZE: f64 = 32.0;
const X_LABEL_AREA_HEIGHT: u32 = 25;
const Y_LABEL_AREA_WIDTH: u32 = 100;
const RELATIVE_PIE_RADIUS: f64 = 0.8;

type PlotColours = Colours<RGBColor>;

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
        colours: Colours,
        width: u32,
        height: u32,
    ) -> anyhow::Result<DynamicImage> {
        let colours = colours.plotters();

        match self {
            Plot::PerDay => per_day(expenses, colours, width, height),
            Plot::Pie => pie(expenses, colours, width, height),
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
    width: u32,
    height: u32,
    colours: PlotColours,
    function: impl for<'root> FnOnce(
        &DrawingArea<BitMapBackend<'root>, Shift>,
        &TextStyle,
    ) -> anyhow::Result<()>,
) -> anyhow::Result<DynamicImage> {
    let mut image_buffer = ImageBuffer::new(width, height);

    // TODO: Move all text to the TUI.
    let text_style = TextStyle {
        font: FontDesc::new(FontFamily::SansSerif, FONT_SIZE, FontStyle::Normal),
        color: colours.text.to_backend_color(),
        // Trying to centre the text here makes it, not centred.
        pos: Pos::new(HPos::Left, VPos::Top),
    };

    let root = BitMapBackend::with_buffer(&mut image_buffer, (width, height)).into_drawing_area();

    root.fill(&colours.background)?;

    function(&root, &text_style)?;

    root.present()?;

    drop(root);

    Ok(DynamicImage::ImageRgb8(image_buffer))
}

fn with_chart<XRange, YRange, Function>(
    width: u32,
    height: u32,
    colours: PlotColours,
    x_range: XRange,
    y_range: YRange,
    function: Function,
) -> anyhow::Result<DynamicImage>
where
    XRange: AsRangedCoord + 'static,
    YRange: AsRangedCoord + 'static,
    XRange::CoordDescType: Ranged + ValueFormatter<<XRange::CoordDescType as Ranged>::ValueType>,
    YRange::CoordDescType: Ranged + ValueFormatter<<YRange::CoordDescType as Ranged>::ValueType>,
    Function: for<'chart> FnOnce(
        &mut ChartContext<
            'chart,
            BitMapBackend<'chart>,
            Cartesian2d<XRange::CoordDescType, YRange::CoordDescType>,
        >,
    ) -> anyhow::Result<()>,
{
    with_root(width, height, colours, move |root, text_style| {
        let mut chart = ChartBuilder::on(root)
            .margin(10)
            .x_label_area_size(X_LABEL_AREA_HEIGHT)
            .y_label_area_size(Y_LABEL_AREA_WIDTH)
            .build_cartesian_2d(x_range, y_range)?;

        chart
            .configure_mesh()
            .axis_style(colours.text)
            .bold_line_style(colours.bold_grid)
            .light_line_style(colours.light_grid)
            .label_style(text_style.clone())
            .draw()?;

        function(&mut chart)?;

        chart
            .configure_series_labels()
            .background_style(colours.background)
            .border_style(colours.border)
            .label_font(text_style.clone())
            .draw()?;

        Ok(())
    })
}

fn per_day<'expenses>(
    expenses: impl IntoIterator<Item = &'expenses Expense>,
    colours: PlotColours,
    width: u32,
    height: u32,
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

    with_chart(
        width,
        height,
        colours,
        start_date..end_date,
        0.0..max_expense,
        move |chart| {
            chart
                .draw_series(LineSeries::new(day_sums, colours.graph))?
                .label("Expenses Per Day")
                .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], colours.graph));

            Ok(())
        },
    )
}

fn pie<'expenses>(
    expenses: impl IntoIterator<Item = &'expenses Expense>,
    colours: PlotColours,
    width: u32,
    height: u32,
) -> anyhow::Result<DynamicImage> {
    fn colour(expense: &Expense) -> RGBColor {
        let mut state = DefaultHasher::new();
        expense.class.hash(&mut state);
        let hash = state.finish();
        let [.., red, green, blue] = hash.to_ne_bytes();
        RGBColor(red, green, blue)
    }

    let image_centre = (width as i32 / 2, height as i32 / 2);
    let radius = min(width, height) as f64 / 2.0 * RELATIVE_PIE_RADIUS;

    let mut slice_classes = Vec::<&str>::new();
    let mut slice_amounts = Vec::new();
    let mut slice_colours = Vec::new();

    for expense in expenses {
        if let Some(index) = slice_classes
            .iter()
            .position(|class| *class == &*expense.class)
        {
            slice_amounts[index] += expense.amount;
        } else {
            slice_classes.push(&expense.class);
            slice_amounts.push(expense.amount);
            slice_colours.push(colour(expense));
        }
    }

    with_root(width, height, colours, |root, text_style| {
        let mut pie = Pie::new(
            &image_centre,
            &radius,
            &slice_amounts,
            &slice_colours,
            &slice_classes,
        );

        pie.label_style(text_style);
        pie.label_offset(75.0);
        pie.percentages(text_style);

        root.draw(&pie)?;

        Ok(())
    })
}
