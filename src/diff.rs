use crate::config::Theme;
use diffy::create_patch;
use ratatui::{
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
};
use std::fs;

pub fn get_file_diff(
    old_root: &str,
    new_root: &str,
    path: &str,
    scroll_offset: usize,
    height: usize,
    theme: &Theme,
) -> (Paragraph<'static>, Paragraph<'static>) {
    let mut rel_path: String = if let Some(stripped) = path.strip_prefix(old_root) {
        stripped.to_string()
    } else if let Some(stripped) = path.strip_prefix(new_root) {
        stripped.to_string()
    } else {
        path.to_string()
    };
    if !rel_path.starts_with('/') {
        rel_path.insert(0, '/');
    }
    let file1 = format!("{}{}", old_root, rel_path);
    let file2 = format!("{}{}", new_root, rel_path);
    let mut old_lines: Vec<Line> = Vec::new();
    let mut new_lines: Vec<Line> = Vec::new();
    let old_file_content = fs::read_to_string(&file1).unwrap_or_else(|_| "".to_string());

    let new_file_content = if file1 != file2 {
        fs::read_to_string(&file2).unwrap_or_else(|_| "".to_string())
    } else {
        old_file_content.clone()
    };
    let lines: Vec<_> = old_file_content.split('\n').collect();
    let patch = create_patch(&old_file_content, &new_file_content);
    let mut current_line_idx = 0;
    for hunk in patch.hunks() {
        let start_of_hunk = hunk.old_range().start().saturating_sub(1);
        while current_line_idx < start_of_hunk {
            let line = Line::from(Span::raw(
                lines
                    .get(current_line_idx)
                    .expect("Unable to get next line of file")
                    .to_string(),
            ));
            old_lines.push(line.clone());
            new_lines.push(line.clone());
            current_line_idx += 1;
        }
        let mut num_modded_lines: i64 = 0;
        for line in hunk.lines() {
            match line {
                diffy::Line::Context(content) => {
                    while num_modded_lines > 0 {
                        old_lines.push(Line::from(Span::styled(
                            " ".to_string(),
                            Style::new().bg(theme.padding),
                        )));
                        num_modded_lines -= 1;
                    }
                    while num_modded_lines < 0 {
                        new_lines.push(Line::from(Span::styled(
                            " ".to_string(),
                            Style::new().bg(theme.padding),
                        )));
                        num_modded_lines += 1;
                    }
                    let line = Line::from(Span::raw(content.to_string()));
                    current_line_idx += 1;
                    old_lines.push(line.clone());
                    new_lines.push(line.clone());
                }
                diffy::Line::Delete(content) => {
                    let line = Line::from(Span::styled(content.to_string(), theme.removed));
                    current_line_idx += 1;
                    old_lines.push(line);
                    num_modded_lines -= 1;
                }
                diffy::Line::Insert(content) => {
                    let line = Line::from(Span::styled(content.to_string(), theme.added));
                    new_lines.push(line);
                    num_modded_lines += 1;
                }
            }
        }

        while num_modded_lines > 0 {
            old_lines.push(Line::from(Span::styled(
                " ".to_string(),
                Style::new().bg(theme.padding),
            )));
            num_modded_lines -= 1;
        }
        while num_modded_lines < 0 {
            new_lines.push(Line::from(Span::styled(
                " ".to_string(),
                Style::new().bg(theme.padding),
            )));
            num_modded_lines += 1;
        }
    }
    let stop = scroll_offset + height;

    old_lines =
        old_lines[scroll_offset.clamp(0, old_lines.len())..stop.clamp(0, old_lines.len())].to_vec();
    new_lines =
        new_lines[scroll_offset.clamp(0, new_lines.len())..stop.clamp(0, new_lines.len())].to_vec();
    (Paragraph::new(old_lines), Paragraph::new(new_lines))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Theme;

    #[test]
    fn test_get_file_diff_unchanged_file() {
        let theme = Theme::default();
        let (_old_p, _new_p) = get_file_diff(
            "test_files/old",
            "test_files/new",
            "test_files/old/unchanged.txt",
            0,
            10,
            &theme,
        );

        use ratatui::buffer::Buffer;
        use ratatui::layout::Rect;
        use ratatui::widgets::Widget;

        let mut old_buf = Buffer::empty(Rect::new(0, 0, 80, 10));
        let mut new_buf = Buffer::empty(Rect::new(0, 0, 80, 10));
        _old_p.render(Rect::new(0, 0, 80, 10), &mut old_buf);
        _new_p.render(Rect::new(0, 0, 80, 10), &mut new_buf);
        assert_eq!(format!("{:?}", old_buf), format!("{:?}", new_buf));
    }

    #[test]
    fn test_get_file_diff_missing_file_old_side() {
        let theme = Theme::default();
        let (old_p, _new_p) = get_file_diff(
            "test_files/old",
            "test_files/new",
            "test_files/old/added_only_new.txt",
            0,
            10,
            &theme,
        );

        use ratatui::buffer::Buffer;
        use ratatui::layout::Rect;
        use ratatui::widgets::Widget;

        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 10));
        old_p.render(Rect::new(0, 0, 80, 10), &mut buf);
        let rendered = format!("{:?}", buf);
        assert!(
            rendered.contains("Error"),
            "old pane should show error for file not in old"
        );
    }

    #[test]
    fn test_get_file_diff_scroll_offset() {
        let theme = Theme::default();
        let (old_p, _new_p) = get_file_diff(
            "test_files/old",
            "test_files/new",
            "test_files/old/many_lines.txt",
            5,
            10,
            &theme,
        );

        use ratatui::buffer::Buffer;
        use ratatui::layout::Rect;
        use ratatui::widgets::Widget;

        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 10));
        old_p.render(Rect::new(0, 0, 80, 10), &mut buf);
        let rendered = format!("{:?}", buf);
        assert!(
            rendered.contains("006"),
            "scrolled content should start around line 006"
        );
    }

    #[test]
    fn test_get_file_diff_binary_file() {
        let theme = Theme::default();
        let (old_p, _new_p) = get_file_diff(
            "test_files/old",
            "test_files/new",
            "test_files/old/binary.bin",
            0,
            10,
            &theme,
        );

        use ratatui::buffer::Buffer;
        use ratatui::layout::Rect;
        use ratatui::widgets::Widget;

        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 10));
        old_p.render(Rect::new(0, 0, 80, 10), &mut buf);
        let rendered = format!("{:?}", buf);
        assert!(
            rendered.contains("Error"),
            "binary file should show an error (non-UTF8)"
        );
    }

    #[test]
    fn test_get_file_diff_modified_file_has_diff() {
        let theme = Theme::default();
        let (old_p, new_p) = get_file_diff(
            "test_files/old",
            "test_files/new",
            "test_files/old/modified.txt",
            0,
            20,
            &theme,
        );

        use ratatui::buffer::Buffer;
        use ratatui::layout::Rect;
        use ratatui::widgets::Widget;

        let mut old_buf = Buffer::empty(Rect::new(0, 0, 80, 20));
        let mut new_buf = Buffer::empty(Rect::new(0, 0, 80, 20));
        old_p.render(Rect::new(0, 0, 80, 20), &mut old_buf);
        new_p.render(Rect::new(0, 0, 80, 20), &mut new_buf);
        let old_rendered = format!("{:?}", old_buf);
        let new_rendered = format!("{:?}", new_buf);
        assert_ne!(
            old_rendered, new_rendered,
            "modified file should produce different old/new panes"
        );
    }
}
