use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    DefaultTerminal, Frame,
    style::{Modifier, Style},
    text::Line,
    widgets::{Block, Borders, List, ListItem, ListState},
};
use std::collections::HashMap;
use std::io;

pub struct TuiState {
    pub old_root: String,
    pub new_root: String,
    pub current_file: Option<String>,
    pub old_files: HashMap<String, Vec<String>>,
    pub new_files: HashMap<String, Vec<String>>,
    pub file_display: Vec<(String, String)>,
    pub bottom_status: String,
    pub file_name_offset: usize,
    pub file_scroll_offset: usize,
    pub state: ListState,
    pub exit: bool,
    pub open_files: Vec<String>,
}

impl TuiState {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        self.state.select_first();
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();

        // === File list panel (left 1/3) ===
        let file_block = Block::bordered()
            .title_top(Line::from("Files"))
            .title_bottom(Line::from(self.bottom_status.clone()))
            .borders(Borders::ALL);
        let mut file_area = area;
        file_area.width /= 3;
        let max_offset = file_area.width as usize + self.file_name_offset;

        let items: Vec<ListItem> = self
            .file_display
            .iter()
            .map(|(_path, name)| {
                let start = self.file_name_offset.min(name.len());
                let end = max_offset.min(name.len());
                ListItem::new(&name[start..end])
            })
            .collect();

        let list = List::new(items)
            .block(file_block)
            .highlight_style(Modifier::REVERSED)
            .highlight_symbol("> ")
            .direction(ratatui::widgets::ListDirection::TopToBottom)
            .style(Style::default());

        frame.render_stateful_widget(list, file_area, &mut self.state);

        // === New panel (top right) ===
        let mut new_area = area;
        new_area.width = area.width * 2 / 3;
        new_area.height = area.height / 2;
        new_area.x = file_area.width;
        let new_block = Block::bordered().title(Line::from("New").centered());

        // === Old panel (bottom right) ===
        let mut old_area = new_area;
        old_area.y = area.height / 2;
        let old_block = Block::bordered()
            .title(Line::from("Old").centered())
            .title_bottom(Line::from(
                "<-/->: Move filenames; k/j: Scroll files; up/down/Enter: choose file",
            ));

        // === Diff content ===
        let path = self.current_file.clone().unwrap_or_default();
        let (old_file, new_file) =
            crate::engine::get_file_diff(self, &path, old_area.height as usize);

        frame.render_widget(old_file.block(old_block).left_aligned(), old_area);
        frame.render_widget(new_file.block(new_block).left_aligned(), new_area);
    }

    fn handle_events(&mut self) -> io::Result<()> {
        if let Event::Key(k) = event::read()? {
            match k.code {
                KeyCode::Esc => {
                    self.exit = true;
                }
                KeyCode::Enter => {
                    self.file_scroll_offset = 0;
                    crate::engine::open_file_or_dir(self);
                }
                KeyCode::Up => {
                    self.state.select_previous();
                }
                KeyCode::Down => {
                    self.state.select_next();
                }
                KeyCode::Right => self.file_name_offset += 1,
                KeyCode::Left => {
                    self.file_name_offset = self.file_name_offset.saturating_sub(1);
                }
                KeyCode::Char('k') | KeyCode::PageUp => {
                    self.file_scroll_offset = self.file_scroll_offset.saturating_sub(1);
                }
                KeyCode::Char('j') | KeyCode::PageDown => {
                    self.file_scroll_offset += 1;
                }
                _ => {}
            }
        }
        Ok(())
    }
}
