mod colours;
mod expense;
mod plot;
mod processing;

use crate::colours::COLOURS;
use crate::expense::Expense;
use crate::plot::Plot;
use anyhow::Context as _;
use clap::Parser;
use enum_iterator::all;
use enum_iterator::next_cycle;
use enum_iterator::previous_cycle;
use rat_salsa::Control;
use rat_salsa::RunConfig;
use rat_salsa::SalsaAppContext;
use rat_salsa::mock;
use rat_salsa::poll::PollCrossterm;
use rat_salsa::run_tui;
use rat_widget::event::ct_event;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::Event;
use ratatui::layout::Constraint;
use ratatui::layout::Layout;
use ratatui::layout::Rect;
use ratatui::widgets::Block;
use ratatui::widgets::StatefulWidget;
use ratatui::widgets::Tabs;
use ratatui::widgets::Widget as _;
use ratatui_image::FilterType;
use ratatui_image::Resize;
use ratatui_image::StatefulImage;
use ratatui_image::picker::Picker;
use std::path::Path;
use std::sync::LazyLock;

type Context = SalsaAppContext<Event, anyhow::Error>;

struct State {
    plot_type: Plot,
    expenses: &'static [Expense],
    picker: Picker,
}

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

    let expenses = Vec::leak(expenses);

    let mut state = State {
        plot_type: Plot::default(),
        expenses,
        picker,
    };

    run_tui(
        mock::init,
        render,
        event,
        error,
        &mut Context::default(),
        &mut state,
        RunConfig::default()?.poll(PollCrossterm),
    )
}

fn render(
    area: Rect,
    buffer: &mut Buffer,
    state: &mut State,
    _: &mut Context,
) -> anyhow::Result<()> {
    let tabs = Tabs::new(all::<Plot>().map(|plot| plot.to_string()))
        .select(all::<Plot>().position(|plot| plot == state.plot_type));

    let block = Block::bordered().title("Expenses Over Time");

    let image = state.plot_type.make_image(state.expenses)?;
    let mut image_state = state.picker.new_resize_protocol(image);

    let [tab_area, content_area] =
        Layout::vertical([Constraint::Max(3), Constraint::Fill(1)]).areas(area);
    let image_area = block.inner(content_area);

    tabs.render(tab_area, buffer);
    block.render(content_area, buffer);
    StatefulImage::new()
        .resize(Resize::Scale(Some(FilterType::Gaussian)))
        .render(image_area, buffer, &mut image_state);

    Ok(())
}

fn event(event: &Event, state: &mut State, _: &mut Context) -> anyhow::Result<Control<Event>> {
    match event {
        ct_event!(key press CONTROL-'q') => Ok(Control::Quit),
        ct_event!(keycode press Tab) | ct_event!(keycode press Right) => {
            state.plot_type = next_cycle(&state.plot_type);
            Ok(Control::Changed)
        }
        ct_event!(keycode press BackTab)
        | ct_event!(keycode press SHIFT-Tab)
        | ct_event!(keycode press Left) => {
            state.plot_type = previous_cycle(&state.plot_type);
            Ok(Control::Changed)
        }
        ct_event!(mouse any for _event) => {
            // TODO: Clicking buttons.
            // TODO: Zooming the graph.
            // TODO: Panning the graph.
            Ok(Control::Continue)
        }
        _ => Ok(Control::Continue),
    }
}

fn error(error: anyhow::Error, _: &mut State, _: &mut Context) -> anyhow::Result<Control<Event>> {
    Err(error)
}
