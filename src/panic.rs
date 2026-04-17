use ratatui::crossterm::ExecutableCommand as _;
use ratatui::crossterm::cursor::DisableBlinking;
use ratatui::crossterm::cursor::SetCursorStyle;
use ratatui::crossterm::event::DisableBracketedPaste;
use ratatui::crossterm::event::DisableMouseCapture;
use ratatui::crossterm::event::PopKeyboardEnhancementFlags;
use ratatui::crossterm::terminal::LeaveAlternateScreen;
use ratatui::crossterm::terminal::disable_raw_mode;
use std::io::stdout;
use std::panic::set_hook;
use std::panic::take_hook;

pub fn set_panic_hook() {
    let old_hook = take_hook();
    set_hook(Box::new(move |info| {
        // Copied from the private function `rat_salsa::framework::shutdown_terminal`.
        let _ = disable_raw_mode();

        let _ = stdout().execute(PopKeyboardEnhancementFlags);
        let _ = stdout().execute(SetCursorStyle::DefaultUserShape);
        let _ = stdout().execute(DisableBlinking);
        let _ = stdout().execute(DisableBracketedPaste);
        let _ = stdout().execute(DisableMouseCapture);
        let _ = stdout().execute(LeaveAlternateScreen);

        old_hook(info);
    }));
}
