use color_eyre::Result;
use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Constraint, Layout, Rect},
    style::{palette::tailwind, Color, Stylize},
    symbols,
    text::Line,
    widgets::{Block, Padding, Paragraph, Tabs, Widget},
    DefaultTerminal,
};
use strum::{Display, EnumIter, FromRepr, IntoEnumIterator};


fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    let mut stdout = io::stdout();
    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture, HideCursor)?;
    Terminal::new(CrosstermBackend::new(stdout))
}

fn teardown_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    let mut stdout = io::stdout();
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(stdout, LeaveAlternateScreen, DisableMouseCapture, ShowCursor)?;
    Ok(())
}

fn main() -> Result<()> {
    let mut terminal = setup_terminal()?;
    run_app(&mut terminal)?;
    teardown_terminal(&mut terminal)?;
    Ok(())
}