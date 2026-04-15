use crate::colours::COLOURS;
use crate::expense::Expense;
use crate::processing::day_sums;
use image::DynamicImage;
use image::ImageBuffer;
use plotters::chart::ChartBuilder;
use plotters::prelude::BitMapBackend;
use plotters::prelude::IntoDrawingArea;
use plotters::prelude::PathElement;
use plotters::series::LineSeries;
use plotters::style::Color as _;
use plotters::style::FontDesc;
use plotters::style::FontFamily;
use plotters::style::FontStyle;
use plotters::style::TextStyle;
use plotters::style::text_anchor::HPos;
use plotters::style::text_anchor::Pos;
use plotters::style::text_anchor::VPos;

pub fn per_day<'expenses>(
    expenses: impl IntoIterator<Item = &'expenses Expense>,
) -> anyhow::Result<DynamicImage> {
    let colours = COLOURS.plotters();

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

    let image_size @ (image_width, image_height) = (1920, 1080);

    let mut image_buffer = ImageBuffer::new(image_width, image_height);

    let text_style = TextStyle {
        font: FontDesc::new(FontFamily::SansSerif, 12.0, FontStyle::Normal),
        color: colours.text.to_backend_color(),
        // Trying to centre the text here makes it, not centred.
        pos: Pos::new(HPos::Left, VPos::Top),
    };

    let root = BitMapBackend::with_buffer(&mut image_buffer, image_size).into_drawing_area();
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

    drop(chart);
    drop(root);

    Ok(DynamicImage::ImageRgb8(image_buffer))
}
