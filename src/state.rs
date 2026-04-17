use crate::Arguments;
use crate::colours::Colours;
use crate::colours::detect_colours;
use crate::expense::Expense;
use crate::processing::years;
use anyhow::Context as _;
use anyhow::bail;
use chrono::Month;
use rat_salsa::SalsaAppContext;
use rat_widget::choice::ChoiceState;
use rat_widget::tabbed::TabbedState;
use ratatui::crossterm::event::Event;
use ratatui_image::picker::Picker;

pub type Context = SalsaAppContext<Event, anyhow::Error>;

pub struct State {
    pub colours: Colours,

    pub expenses: &'static [Expense],
    pub picker: Picker,

    pub plot_tabs: TabbedState,
    pub year_input: ChoiceState<i32>,
    pub month_input: ChoiceState<Option<Month>>,
}

impl State {
    pub fn new(arguments: Arguments) -> anyhow::Result<State> {
        let colours = detect_colours()?;

        let mut picker = Picker::from_query_stdio()?;
        picker.set_background_color(colours.rgba().background);

        let expenses: &[_] = csv::Reader::from_path(&arguments.path)
            .with_context(|| format!("reading file: `{}`", arguments.path.display()))?
            .deserialize::<Expense>()
            .collect::<Result<Vec<_>, _>>()?
            .leak();

        if expenses.is_empty() {
            bail!("there are no expenses to analyse");
        }

        let mut year_input = ChoiceState::new();
        year_input.core.set_value(*years(expenses).last().unwrap());

        Ok(State {
            colours,

            expenses,
            picker,

            plot_tabs: TabbedState::new(),
            year_input,
            month_input: ChoiceState::new(),
        })
    }
}
