mod colours;
mod expense;
mod processing;

use crate::colours::colours;
use crate::expense::Expense;
use crate::processing::day_sums;
use anyhow::Context;
use clap::Parser;
use plotters::chart::ChartBuilder;
use plotters::prelude::BitMapBackend;
use plotters::prelude::IntoDrawingArea;
use plotters::prelude::PathElement;
use plotters::series::LineSeries;
use plotters::style::Color;
use plotters::style::FontDesc;
use plotters::style::FontFamily;
use plotters::style::FontStyle;
use plotters::style::TextStyle;
use plotters::style::text_anchor::HPos;
use plotters::style::text_anchor::Pos;
use plotters::style::text_anchor::VPos;
use std::path::Path;

#[derive(Parser)]
struct Arguments {
    path: Box<Path>,
}

fn main() -> anyhow::Result<()> {
    let arguments = Arguments::parse();
    let colours = colours()?;

    let expenses: Vec<_> = csv::Reader::from_path(&arguments.path)
        .with_context(|| format!("reading file: `{}`", arguments.path.display()))?
        .deserialize::<Expense>()
        .collect::<Result<_, _>>()?;

    let day_sums = day_sums(&expenses);

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

    let text_style = TextStyle {
        font: FontDesc::new(FontFamily::SansSerif, 12.0, FontStyle::Normal),
        color: colours.text.to_backend_color(),
        // Trying to centre the text here makes it, not centred.
        pos: Pos::new(HPos::Left, VPos::Top),
    };

    let root = BitMapBackend::new("0.png", (1920, 1080)).into_drawing_area();
    root.fill(&colours.background)?;
    let mut chart = ChartBuilder::on(&root)
        .caption("Expenses Over Time", text_style.clone())
        .margin(10)
        .x_label_area_size(25)
        .y_label_area_size(50)
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

    root.present()?;

    Ok(())
}
