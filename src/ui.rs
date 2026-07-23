use crate::render;
use crate::state::TuiState;
use crossterm::event::{self, Event, KeyCode};
use ratatui::DefaultTerminal;

pub fn run(state: &mut TuiState, terminal: &mut DefaultTerminal) -> anyhow::Result<()> {
    state.state.select_first();
    while !state.exit {
        terminal.draw(|frame| render::draw(frame, state))?;
        handle_events(state)?;
    }
    Ok(())
}

fn handle_events(state: &mut TuiState) -> std::io::Result<()> {
    if let Event::Key(k) = event::read()? {
        match k.code {
            KeyCode::Esc => {
                state.exit = true;
            }
            KeyCode::Enter => {
                state.file_scroll_offset = 0;
                state.open_file_or_dir();
            }
            KeyCode::Up => {
                state.state.select_previous();
            }
            KeyCode::Down => {
                state.state.select_next();
            }
            KeyCode::Right => state.file_name_offset += 1,
            KeyCode::Left => {
                state.file_name_offset = state.file_name_offset.saturating_sub(1);
            }
            KeyCode::Char('k') | KeyCode::PageUp => {
                state.file_scroll_offset = state.file_scroll_offset.saturating_sub(1);
            }
            KeyCode::Char('j') | KeyCode::PageDown => {
                state.file_scroll_offset += 1;
            }
            KeyCode::Tab => state.hide_sidebar = !state.hide_sidebar,
            _ => {}
        }
    }
    Ok(())
}
