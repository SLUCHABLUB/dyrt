mod colours;
mod expense;
mod panic;
mod plot;
mod processing;

use crate::colours::COLOURS;
use crate::expense::Expense;
use crate::panic::set_panic_hook;
use crate::plot::Plot;
use crate::processing::filter_to_period;
use crate::processing::months;
use crate::processing::years;
use anyhow::Context as _;
use anyhow::bail;
use chrono::Month;
use clap::Parser;
use enum_iterator::all;
use itertools::chain;
use rat_salsa::Control;
use rat_salsa::RunConfig;
use rat_salsa::SalsaAppContext;
use rat_salsa::mock;
use rat_salsa::poll::PollCrossterm;
use rat_salsa::run_tui;
use rat_widget::choice::Choice;
use rat_widget::choice::ChoiceState;
use rat_widget::event::HandleEvent;
use rat_widget::event::MouseOnly;
use rat_widget::event::Regular;
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
use ratatui_image::Image;
use ratatui_image::Resize;
use ratatui_image::picker::Picker;
use std::iter::once;
use std::path::Path;

type Context = SalsaAppContext<Event, anyhow::Error>;

struct State {
    expenses: &'static [Expense],
    picker: Picker,

    plot_tabs: TabbedState,
    year_input: ChoiceState<i32>,
    month_input: ChoiceState<Option<Month>>,
}

#[derive(Parser)]
struct Arguments {
    path: Box<Path>,
}

fn main() -> anyhow::Result<()> {
    let arguments = Arguments::parse();

    let mut picker = Picker::from_query_stdio()?;
    picker.set_background_color(COLOURS.rgba().background);

    let expenses: &[_] = csv::Reader::from_path(&arguments.path)
        .with_context(|| format!("reading file: `{}`", arguments.path.display()))?
        .deserialize::<Expense>()
        .collect::<Result<Vec<_>, _>>()?
        .leak();

    if expenses.is_empty() {
        bail!("there are no expenses to analyse");
    }

    let mut state = State {
        expenses,
        picker,

        plot_tabs: TabbedState::new(),
        year_input: ChoiceState::new(),
        month_input: ChoiceState::new(),
    };

    state
        .year_input
        .core
        .set_value(*years(expenses).last().unwrap());

    set_panic_hook();

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

    let months = months(state.expenses, state.year_input.core.value());
    let years = years(state.expenses);

    // Create Widgets

    let tabs = Tabbed::new()
        .tab_type(TabType::Glued)
        .tabs(all::<Plot>().map(|plot| plot.to_string()));

    let (year_input, year_input_popup) = Choice::new()
        .items(years.iter().map(|year| (*year, year.to_string())))
        .into_widgets();
    let (month_input, month_input_popup) = Choice::new()
        .items(chain(
            once((None, "All year")),
            months.iter().map(|month| (Some(*month), month.name())),
        ))
        .into_widgets();

    let block = Block::bordered().title(plot_type.title());

    let expenses = filter_to_period(
        state.expenses,
        state.year_input.core.value(),
        state.month_input.core.value(),
    );

    // Calculate Areas

    const CHOICE_PADDING: u16 = 3;

    let [bar_area, content_area] =
        Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(area);
    let [tab_area, year_input_area, month_input_area] = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length(4 + CHOICE_PADDING),
        Constraint::Length(9 + CHOICE_PADDING),
    ])
    .areas(bar_area);
    let image_area = block.inner(content_area);

    // Create Area-Dependent Widgets

    let (character_width, character_height) = state.picker.font_size();

    let image = plot_type.make_image(
        expenses,
        character_width as u32 * image_area.width as u32,
        character_height as u32 * image_area.height as u32,
    )?;
    let protocol =
        state
            .picker
            .new_protocol(image, image_area, Resize::Scale(Some(FilterType::Gaussian)))?;

    let image = Image::new(&protocol);

    // Render Widgets

    tabs.render(tab_area, buffer, &mut state.plot_tabs);

    year_input.render(year_input_area, buffer, &mut state.year_input);
    month_input.render(month_input_area, buffer, &mut state.month_input);

    block.render(content_area, buffer);
    image.render(image_area, buffer);

    // Render Popups

    year_input_popup.render(area, buffer, &mut state.year_input);
    month_input_popup.render(area, buffer, &mut state.month_input);

    Ok(())
}

fn event(event: &Event, state: &mut State, _: &mut Context) -> anyhow::Result<Control<Event>> {
    match event {
        ct_event!(key press CONTROL-'q') => return Ok(Control::Quit),
        ct_event!(resized) => return Ok(Control::Changed),
        _ => (),
    }

    event_flow!(state.year_input.handle(event, Regular));
    event_flow!(state.month_input.handle(event, Regular));
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
