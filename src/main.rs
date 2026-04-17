mod colours;
mod expense;
mod plot;
mod processing;

use crate::colours::COLOURS;
use crate::expense::Expense;
use crate::plot::Plot;
use anyhow::Context as _;
use anyhow::bail;
use chrono::Month;
use clap::Parser;
use enum_iterator::all;
use rat_salsa::Control;
use rat_salsa::RunConfig;
use rat_salsa::SalsaAppContext;
use rat_salsa::mock;
use rat_salsa::poll::PollCrossterm;
use rat_salsa::run_tui;
use rat_widget::choice::ChoiceState;
use rat_widget::event::HandleEvent;
use rat_widget::event::MouseOnly;
use rat_widget::event::TabbedOutcome;
use rat_widget::event::ct_event;
use rat_widget::event::event_flow;
use rat_widget::tabbed::TabType;
use rat_widget::tabbed::Tabbed;
use rat_widget::tabbed::TabbedState;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::Event;
use ratatui::layout::Constraint;
use ratatui::layout::Layout;
use ratatui::layout::Rect;
use ratatui::widgets::Block;
use ratatui::widgets::StatefulWidget;
use ratatui::widgets::Widget as _;
use ratatui_image::FilterType;
use ratatui_image::Resize;
use ratatui_image::StatefulImage;
use ratatui_image::picker::Picker;
use std::path::Path;
use std::sync::LazyLock;

type Context = SalsaAppContext<Event, anyhow::Error>;

struct State {
    expenses: &'static [Expense],
    picker: Picker,

    plot_tabs: TabbedState,
    _year_input: ChoiceState<i32>,
    _month_input: ChoiceState<Option<Month>>,
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

    let expenses = csv::Reader::from_path(&arguments.path)
        .with_context(|| format!("reading file: `{}`", arguments.path.display()))?
        .deserialize::<Expense>()
        .collect::<Result<_, _>>()?;

    let expenses = Vec::leak(expenses);

    if expenses.is_empty() {
        bail!("there are no expenses to analyse");
    }

    let mut state = State {
        expenses,
        picker,

        plot_tabs: TabbedState::new(),
        _year_input: ChoiceState::new(),
        _month_input: ChoiceState::new(),
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
    let plot_type = all::<Plot>()
        .nth(state.plot_tabs.selected().unwrap_or(0))
        .unwrap();

    let tabs = Tabbed::new()
        .tab_type(TabType::Glued)
        .tabs(all::<Plot>().map(|plot| plot.to_string()));

    let block = Block::bordered().title(plot_type.title());

    let image = plot_type.make_image(state.expenses)?;
    let mut image_state = state.picker.new_resize_protocol(image);

    let [bar_area, content_area] =
        Layout::vertical([Constraint::Max(1), Constraint::Fill(1)]).areas(area);
    let [tab_area, _period_input_area] =
        Layout::horizontal([Constraint::Fill(1); 2]).areas(bar_area);
    let image_area = block.inner(content_area);

    tabs.render(tab_area, buffer, &mut state.plot_tabs);
    block.render(content_area, buffer);
    StatefulImage::new()
        .resize(Resize::Scale(Some(FilterType::Gaussian)))
        .render(image_area, buffer, &mut image_state);

    Ok(())
}

fn event(event: &Event, state: &mut State, _: &mut Context) -> anyhow::Result<Control<Event>> {
    if matches!(event, ct_event!(key press CONTROL-'q')) {
        return Ok(Control::Quit);
    }

    event_flow!(handle_tab_event(event, &mut state.plot_tabs));

    // TODO: Zooming and panning the graph.

    Ok(Control::Continue)
}

fn handle_tab_event(event: &Event, state: &mut TabbedState) -> TabbedOutcome {
    fn cycle_tab(state: &mut TabbedState, by: isize) -> TabbedOutcome {
        let tab_count = state.tab_title_areas.len() as isize;
        let selected = state.selected().unwrap_or(0) as isize;
        let new = (selected + by).rem_euclid(tab_count);

        state.select(Some(new as usize));

        TabbedOutcome::Changed
    }

    event_flow!(return state.handle(event, MouseOnly));

    match event {
        ct_event!(keycode press Tab) => cycle_tab(state, 1),
        ct_event!(keycode press BackTab) | ct_event!(keycode press SHIFT-Tab) => {
            cycle_tab(state, -1)
        }
        _ => TabbedOutcome::Continue,
    }
}

fn error(error: anyhow::Error, _: &mut State, _: &mut Context) -> anyhow::Result<Control<Event>> {
    Err(error)
}
