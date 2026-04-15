mod colours;
mod expense;
mod plot;
mod processing;

use crate::colours::COLOURS;
use crate::expense::Expense;
use anyhow::Context;
use clap::Parser;
use ratatui::DefaultTerminal;
use ratatui::crossterm::event::Event;
use ratatui::crossterm::event::KeyCode;
use ratatui::crossterm::event::KeyModifiers;
use ratatui::crossterm::event::poll;
use ratatui::crossterm::event::read;
use ratatui::run;
use ratatui::widgets::Block;
use ratatui_image::FilterType;
use ratatui_image::Resize;
use ratatui_image::StatefulImage;
use ratatui_image::picker::Picker;
use ratatui_image::protocol::StatefulProtocol;
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

    let plot = plot::per_day(&expenses)?;
    let mut image_state = picker.new_resize_protocol(plot);

    let mut should_redraw = true;

    run(|terminal| {
        loop {
            if should_redraw {
                should_redraw = false;
                redraw(terminal, &mut image_state)?;
            }

            while poll(Duration::ZERO)? {
                let event = read()?;

                match event {
                    Event::Key(event) => {
                        if event.is_press()
                            && event.modifiers == KeyModifiers::CONTROL
                            && event.code == KeyCode::Char('q')
                        {
                            return anyhow::Ok(());
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
    image_state: &mut StatefulProtocol,
) -> anyhow::Result<()> {
    terminal.draw(|frame| {
        let block = Block::bordered().title("Expenses Over Time");

        frame.render_stateful_widget(
            StatefulImage::new().resize(Resize::Scale(Some(FilterType::Gaussian))),
            block.inner(frame.area()),
            image_state,
        );
        frame.render_widget(block, frame.area());
    })?;

    Ok(())
}
