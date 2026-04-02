use crossterm::event::{self, Event, KeyCode};
use diffy::create_patch;
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, StatefulWidget, Widget},
};
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::{
    fs::{self},
    io::{self},
};

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
    pub terminal_size: ratatui::prelude::Size,
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
    fn draw(&self, frame: &mut Frame) {
        let mut state = self.state;
        frame.render_stateful_widget(self, frame.area(), &mut state);
    }
    fn handle_events(&mut self) -> io::Result<()> {
        if let Event::Key(k) = event::read()? {
            match k.code {
                KeyCode::Esc => {
                    self.exit = true;
                }
                KeyCode::Enter => {
                    self.file_scroll_offset = 0;
                    self.open_file_or_dir();
                }
                KeyCode::Up => {
                    self.state.select_previous();
                }
                KeyCode::Down => {
                    self.state.select_next();
                }
                KeyCode::Right => self.file_name_offset += 1,
                KeyCode::Left => {
                    if self.file_name_offset > 0 {
                        self.file_name_offset -= 1;
                    }
                }
                KeyCode::PageUp => {
                    if self.file_scroll_offset > 0 {
                        self.file_scroll_offset -= 1;
                    }
                }
                KeyCode::PageDown => {
                    self.file_scroll_offset += 1;
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn get_joined_paths(&mut self, path: &String) -> Vec<String> {
        let tmpvec = Vec::<String>::new();
        let x = self
            .old_files
            .get(path)
            .cloned()
            .unwrap_or_else(|| tmpvec.clone());
        let y = self.new_files.get(path).cloned().unwrap_or(tmpvec);
        let mut result = x;
        result.extend(y);
        result = result
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>();
        result.par_sort_unstable();
        result.reverse();
        result
    }
    fn close_dir(&mut self, path: &String, children: &Vec<String>) {
        for x in children {
            let mut full_path = path.clone();
            full_path.push('/');
            full_path.push_str(x.clone().trim_start());
            if self.open_files.contains(&full_path) {
                let child_dirs = self.get_joined_paths(&full_path);
                self.close_dir(&full_path, &child_dirs);
            }
        }
        self.file_display.retain(|(pth, file)| {
            !children
                .clone()
                .iter()
                .map(|x| x.trim_start().to_string())
                .collect::<Vec<String>>()
                .contains(&file.clone().trim_start().to_string())
        });
        let ind = self
            .open_files
            .iter()
            .position(|x| x == path)
            .expect("Failed to find path in open files");
        self.open_files.remove(ind);
    }
    fn open_file_or_dir(&mut self) {
        let selected = self.state.selected().unwrap_or(0);
        let tmpval = ("".to_string(), "".to_string());
        let current_dir = self.file_display.get(selected).unwrap_or(&tmpval);
        let mut full_path: String = current_dir.clone().0.to_string();
        full_path.push('/');
        full_path.push_str(current_dir.1.clone().trim_start());
        let mut children = self.get_joined_paths(&full_path);
        let mut seen = HashSet::new();
        children.retain(|x| seen.insert(x.clone()));
        if children.len() != 0 {
            if self.open_files.contains(&full_path) {
                self.close_dir(&full_path, &children);
            } else {
                for x in children {
                    let mut tmp_display = "".to_string();
                    let fp_temp =
                        full_path
                            .replacen(&self.old_root, "", 1)
                            .replacen(&self.new_root, "", 1);
                    for _y in fp_temp.match_indices("/") {
                        tmp_display.push_str("  ");
                    }
                    tmp_display.push_str(&x);
                    self.file_display
                        .insert(selected + 1, (full_path.clone(), tmp_display));
                }
                self.open_files.push(full_path);
            }
            return;
        }
        self.current_file = Some(full_path);
    }
    fn get_file_diff(&self, path: &String, height: usize) -> (Paragraph<'_>, Paragraph<'_>) {
        let mut rel_path: String = if path.starts_with(&self.old_root) {
            path[self.old_root.len()..].to_string()
        } else if path.starts_with(&self.new_root) {
            path[self.new_root.len()..].to_string()
        } else {
            path.to_string()
        };
        rel_path = if !rel_path.starts_with('/') {
            format!("{}{}", "/", rel_path)
        } else {
            rel_path.to_string()
        };
        let mut file1 = self.old_root.clone();
        file1.push_str(&rel_path);
        let mut file2 = self.new_root.clone();
        file2.push_str(&rel_path);
        let mut old_lines: Vec<Line> = Vec::new();
        let mut new_lines: Vec<Line> = Vec::new();
        let new_file_content = fs::read_to_string(&file1)
            .unwrap_or_else(|e| format!("Error reading file {}:\n{}", &file1, e));

        let old_file_content = if file1 != file2 {
            fs::read_to_string(&file2)
                .unwrap_or_else(|e| format!("Error reading file {}:\n{}", &file1, e))
        } else {
            new_file_content.clone()
        };
        let old_file_content = fs::read_to_string(&file2)
            .unwrap_or_else(|e| format!("Error reading file {}:\n{}", &file1, e));

        let patch = create_patch(&old_file_content, &new_file_content);
        for hunk in patch.hunks() {
            for line in hunk.lines() {
                match line {
                    diffy::Line::Context(content) => {
                        let line = Line::from(Span::raw(content.to_string()));
                        old_lines.push(line.clone());
                        new_lines.push(line.clone());
                    }
                    diffy::Line::Delete(content) => {
                        let line = Line::from(Span::styled(
                            content.to_string(),
                            ratatui::style::Color::Red,
                        ));
                        old_lines.push(line);
                        new_lines.push(Line::from(Span::raw("".to_string())));
                    }
                    diffy::Line::Insert(content) => {
                        let line = Line::from(Span::styled(
                            content.to_string(),
                            ratatui::style::Color::Green,
                        ));
                        new_lines.push(line);
                        old_lines.push(Line::from(Span::raw("".to_string())));
                    }
                }
            }
        }
        let stop = self.file_scroll_offset + height;

        old_lines = old_lines
            [self.file_scroll_offset.clamp(0, old_lines.len())..stop.clamp(0, old_lines.len())]
            .to_vec();
        new_lines = new_lines
            [self.file_scroll_offset.clamp(0, new_lines.len())..stop.clamp(0, new_lines.len())]
            .to_vec();
        (Paragraph::new(old_lines), Paragraph::new(new_lines))
    }
}
impl StatefulWidget for &TuiState {
    type State = ListState;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let file_title = Line::from("Files");
        let file_block = Block::bordered()
            .title_top(file_title)
            .title_bottom(Line::from(self.bottom_status.to_string()))
            .borders(Borders::ALL);
        let mut file_area = area;
        file_area.width /= 3;
        let max_size = file_area.width as usize + self.file_name_offset;
        let tmp_array: Vec<ListItem> = self
            .file_display
            .par_iter()
            .map(|(_path, y): &(String, String)| {
                ListItem::new(
                    &y[self.file_name_offset.clamp(0, y.len())
                        ..max_size.clamp(1, y.len().clamp(1, y.len()))],
                )
            })
            .collect();
        let list: List = List::new(tmp_array)
            .block(file_block)
            .highlight_style(Modifier::REVERSED)
            .highlight_symbol("> ")
            .direction(ratatui::widgets::ListDirection::TopToBottom)
            .style(Style::default());

        ratatui::prelude::StatefulWidget::render(list, file_area, buf, state);

        let new_title = Line::from("New");
        let new_block = Block::bordered().title(new_title.centered());
        let mut new_area = area;
        new_area.width *= 2;
        new_area.width /= 3;
        new_area.height = area.height / 2;
        new_area.x += file_area.width;

        let old_title = Line::from("Old");
        let old_block = Block::bordered().title(old_title.centered()).title_bottom(
            "<-/->: Move filenames; pgUp/pgDn: Scroll files; up/down/Enter: choose file"
                .to_string(),
        );
        let mut old_area = new_area;
        old_area.y += file_area.height / 2;
        let path = self.current_file.clone().unwrap_or("".to_string());
        let (old_file, new_file) = self.get_file_diff(&path, old_area.height as usize);
        old_file
            .block(old_block)
            .left_aligned()
            .render(old_area, buf);
        new_file
            .block(new_block)
            .left_aligned()
            .render(new_area, buf);
    }
}
