use crate::diff;
use crate::state::TuiState;
use ratatui::{
    Frame,
    style::{Modifier, Style},
    text::Line,
    widgets::{Block, Borders, List, ListItem},
};

pub fn slice_display_name(name: &str, offset: usize, max_offset: usize) -> &str {
    let start = offset.min(name.len());
    let end = max_offset.min(name.len());
    let mut indices = name.char_indices().map(|(i, _)| i);
    let byte_start = indices.nth(start).unwrap_or(name.len());
    let byte_end = indices
        .nth(end.saturating_sub(start + 1))
        .unwrap_or(name.len());
    &name[byte_start..byte_end]
}

pub fn draw(frame: &mut Frame, state: &mut TuiState) {
    let area = frame.area();

    let mut file_area = area;
    file_area.width /= 3;
    if !state.hide_sidebar {
        let file_block = Block::bordered()
            .title_top(Line::from("Files"))
            .title_bottom(Line::from(state.bottom_status.to_string()))
            .borders(Borders::ALL);
        let max_offset = file_area.width as usize + state.file_name_offset;

        let items: Vec<ListItem> = state
            .file_display
            .iter()
            .map(|(_path, name)| {
                ListItem::new(slice_display_name(name, state.file_name_offset, max_offset))
            })
            .collect();

        let list = List::new(items)
            .block(file_block)
            .highlight_style(Modifier::REVERSED)
            .highlight_symbol("> ")
            .direction(ratatui::widgets::ListDirection::TopToBottom)
            .style(Style::default());

        frame.render_stateful_widget(list, file_area, &mut state.state);
    }

    let mut new_area = area;
    if !state.hide_sidebar {
        new_area.width = area.width * 2 / 3;
        new_area.height = area.height / 2;
        new_area.x = file_area.width;
    } else {
        new_area.width = area.width / 2;
        new_area.height = area.height;
        new_area.x = 0; //file_area.width;
    }
    let new_block = Block::bordered().title(Line::from("New").centered());

    let mut old_area = new_area;
    if state.hide_sidebar {
        old_area.x = area.width / 2;
    } else {
        old_area.y = area.height / 2;
    }
    let old_block = Block::bordered()
        .title(Line::from("Old").centered())
        .title_bottom(Line::from(
            "<-/->: Move filenames; k/j: Scroll files; up/down/Enter: choose file",
        ));

    let path = state.current_file.clone().unwrap_or_default();
    let height = old_area.height as usize;
    let (old_file, new_file) = diff::get_file_diff(
        &state.old_root,
        &state.new_root,
        &path,
        state.file_scroll_offset,
        height,
        &state.theme,
    );

    frame.render_widget(old_file.block(old_block).left_aligned(), old_area);
    frame.render_widget(new_file.block(new_block).left_aligned(), new_area);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slice_full_name_when_zero_offset() {
        assert_eq!(slice_display_name("hello.txt", 0, 9), "hello.txt");
        assert_eq!(slice_display_name("中文.txt", 0, 8), "中文.txt");
    }

    #[test]
    fn test_slice_max_offset_beyond_length() {
        assert_eq!(slice_display_name("hello.txt", 0, 100), "hello.txt");
        assert_eq!(slice_display_name("hello.txt", 2, 100), "llo.txt");
    }

    #[test]
    fn test_slice_offset_beyond_name_length_returns_empty() {
        assert_eq!(slice_display_name("hello.txt", 50, 60), "");
        assert_eq!(slice_display_name("short", 100, 150), "");
    }

    #[test]
    fn test_slice_offset_equal_to_length_returns_empty() {
        assert_eq!(slice_display_name("hello.txt", 9, 9), "");
    }

    #[test]
    fn test_slice_mid_ascii_name() {
        assert_eq!(slice_display_name("abcdefghij", 3, 7), "defg");
    }

    #[test]
    fn test_slice_unicode_stays_on_char_boundaries() {
        assert_eq!(slice_display_name("中文目录文件.txt", 1, 3), "文目");
        assert_eq!(slice_display_name("日本語", 1, 100), "本語");
    }

    #[test]
    fn test_slice_empty_name() {
        assert_eq!(slice_display_name("", 0, 10), "");
        assert_eq!(slice_display_name("", 5, 5), "");
    }
}
