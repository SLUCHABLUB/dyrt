mod colours;
mod expense;
mod plot;
mod processing;

use crate::colours::COLOURS;
use crate::expense::Expense;
use crate::plot::Plot;
use anyhow::Context;
use clap::Parser;
use enum_iterator::all;
use enum_iterator::next_cycle;
use enum_iterator::previous_cycle;
use ratatui::DefaultTerminal;
use ratatui::crossterm::event::Event;
use ratatui::crossterm::event::KeyCode;
use ratatui::crossterm::event::KeyModifiers;
use ratatui::crossterm::event::poll;
use ratatui::crossterm::event::read;
use ratatui::layout::Constraint;
use ratatui::layout::Layout;
use ratatui::run;
use ratatui::widgets::Block;
use ratatui::widgets::Tabs;
use ratatui_image::FilterType;
use ratatui_image::Resize;
use ratatui_image::StatefulImage;
use ratatui_image::picker::Picker;
use std::path::Path;
use std::sync::LazyLock;
use std::time::Duration;

#[derive(Parser)]
struct Arguments {
    path: Box<Path>,
}

fn main() -> anyhow::Result<()> {
    let arguments = Arguments::parse();
    LazyLock::force(&COLOURS);
    let mut picker = Picker::from_query_stdio()?;
    picker.set_background_color(COLOURS.rgba().background);

    let expenses: Vec<_> = csv::Reader::from_path(&arguments.path)
        .with_context(|| format!("reading file: `{}`", arguments.path.display()))?
        .deserialize::<Expense>()
        .collect::<Result<_, _>>()?;

    let mut plot_type = Plot::default();

    let mut should_redraw = true;

    run(|terminal| {
        loop {
            if should_redraw {
                should_redraw = false;
                redraw(terminal, &expenses, &picker, plot_type)?;
            }

            while poll(Duration::ZERO)? {
                let event = read()?;

                match event {
                    Event::Key(event) => {
                        if !event.is_press() {
                            continue;
                        }

                        match (event.modifiers, event.code) {
                            (KeyModifiers::CONTROL, KeyCode::Char('q')) => return anyhow::Ok(()),
                            (KeyModifiers::NONE, KeyCode::Tab | KeyCode::Right) => {
                                plot_type = next_cycle(&plot_type);
                                should_redraw = true;
                            }
                            (KeyModifiers::SHIFT, KeyCode::Tab)
                            | (KeyModifiers::NONE, KeyCode::BackTab | KeyCode::Left) => {
                                plot_type = previous_cycle(&plot_type);
                                should_redraw = true;
                            }
                            _ => (),
                        }
                    }
                    Event::Mouse(_event) => {
                        // TODO: Clicking buttons.
                        // TODO: Zooming the graph.
                        // TODO: Panning the graph.
                    }
                    Event::Resize(_, _) => should_redraw = true,
                    _ => (),
                }
            }
        }
    })?;

    Ok(())
}

fn redraw(
    terminal: &mut DefaultTerminal,
    expenses: &[Expense],
    picker: &Picker,
    plot_type: Plot,
) -> anyhow::Result<()> {
    let tabs = Tabs::new(all::<Plot>().map(|plot| plot.to_string()))
        .select(all::<Plot>().position(|plot| plot == plot_type));

    let block = Block::bordered().title("Expenses Over Time");

    let image = plot_type.make_image(expenses)?;
    let mut image_state = picker.new_resize_protocol(image);

    terminal.draw(|frame| {
        let area = frame.area();
        let [tab_area, content_area] =
            Layout::vertical([Constraint::Max(3), Constraint::Fill(1)]).areas(area);
        let image_area = block.inner(content_area);

        frame.render_widget(tabs, tab_area);
        frame.render_widget(block, content_area);
        frame.render_stateful_widget(
            StatefulImage::new().resize(Resize::Scale(Some(FilterType::Gaussian))),
            image_area,
            &mut image_state,
        );
    })?;

    Ok(())
}
