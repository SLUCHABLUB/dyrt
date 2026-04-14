mod expense;
mod processing;

use crate::expense::Expense;
use crate::processing::day_sums;
use anyhow::Context;
use clap::Parser;
use plotters::chart::ChartBuilder;
use plotters::prelude::BitMapBackend;
use plotters::prelude::IntoDrawingArea;
use plotters::prelude::PathElement;
use plotters::series::LineSeries;
use plotters::style::BLACK;
use plotters::style::Color;
use plotters::style::FontDesc;
use plotters::style::FontFamily;
use plotters::style::FontStyle;
use plotters::style::RED;
use plotters::style::TextStyle;
use plotters::style::WHITE;
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
        color: BLACK.to_backend_color(),
        pos: Pos::new(HPos::Center, VPos::Center),
    };

    let root = BitMapBackend::new("0.png", (1920, 1080)).into_drawing_area();
    root.fill(&WHITE)?;
    let mut chart = ChartBuilder::on(&root)
        .caption("Expenses Over Time", text_style)
        .margin(10)
        .x_label_area_size(25)
        .y_label_area_size(50)
        .build_cartesian_2d(start_date..end_date, 0.0..max_expense)?;

    chart.configure_mesh().draw()?;

    chart
        .draw_series(LineSeries::new(day_sums, &RED))?
        .label("Expenses Per Day")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], RED));

    chart
        .configure_series_labels()
        .background_style(WHITE)
        .border_style(BLACK)
        .draw()?;

    root.present()?;

    Ok(())
}
