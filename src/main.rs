mod colours;
mod expense;
mod plot;
mod processing;

use crate::colours::COLOURS;
use crate::expense::Expense;
use anyhow::Context;
use clap::Parser;
use ratatui::crossterm::event::Event;
use ratatui::crossterm::event::KeyCode;
use ratatui::crossterm::event::KeyModifiers;
use ratatui::crossterm::event::poll;
use ratatui::crossterm::event::read;
use ratatui::run;
use ratatui::widgets::Block;
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
    let picker = Picker::from_query_stdio()?;

    let expenses: Vec<_> = csv::Reader::from_path(&arguments.path)
        .with_context(|| format!("reading file: `{}`", arguments.path.display()))?
        .deserialize::<Expense>()
        .collect::<Result<_, _>>()?;

    let plot = plot::per_day(&expenses)?;
    let mut image_state = picker.new_resize_protocol(plot);

    run(|terminal| {
        loop {
            terminal.draw(|frame| {
                let block = Block::bordered().title("Expenses Over Time");

                frame.render_stateful_widget(
                    StatefulImage::new().resize(Resize::Scale(None)),
                    block.inner(frame.area()),
                    &mut image_state,
                );
                frame.render_widget(block, frame.area());
            })?;

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
                    _ => (),
                }
            }
        }
    })?;

    Ok(())
}
