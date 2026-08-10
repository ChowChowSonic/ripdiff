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
    let patch = create_patch(&old_file_content, &new_file_content);
    let stop = scroll_offset + height;
    if patch.hunks().is_empty() {
        old_lines = old_file_content
            .split("\n")
            .map(|x| Line::from(Span::styled(x.to_string(), Style::new())))
            .collect();
        old_lines = old_lines
            [scroll_offset.clamp(0, old_lines.len())..stop.clamp(0, old_lines.len())]
            .to_vec();
        new_lines = new_file_content
            .split("\n")
            .map(|x| Line::from(Span::styled(x.to_string(), Style::new())))
            .collect();
        new_lines = new_lines
            [scroll_offset.clamp(0, new_lines.len())..stop.clamp(0, new_lines.len())]
            .to_vec();
        return (Paragraph::new(old_lines), Paragraph::new(new_lines));
    }
    let lines: Vec<_> = old_file_content.split('\n').collect();
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
    use ratatui::style::Color;

    const OLD_DIR: &str = "test_files/old";
    const NEW_DIR: &str = "test_files/new";

    #[test]
    fn test_path_starts_with_old_root() {
        let theme = Theme::default();
        let (old_p, new_p) = get_file_diff(
            OLD_DIR,
            NEW_DIR,
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
        old_p.render(Rect::new(0, 0, 80, 10), &mut old_buf);
        new_p.render(Rect::new(0, 0, 80, 10), &mut new_buf);
        assert_eq!(format!("{:?}", old_buf), format!("{:?}", new_buf));
    }

    #[test]
    fn test_path_starts_with_new_root() {
        let theme = Theme::default();
        let (old_p, new_p) = get_file_diff(
            OLD_DIR,
            NEW_DIR,
            "test_files/new/unchanged.txt",
            0,
            10,
            &theme,
        );

        use ratatui::buffer::Buffer;
        use ratatui::layout::Rect;
        use ratatui::widgets::Widget;

        let mut old_buf = Buffer::empty(Rect::new(0, 0, 80, 10));
        let mut new_buf = Buffer::empty(Rect::new(0, 0, 80, 10));
        old_p.render(Rect::new(0, 0, 80, 10), &mut old_buf);
        new_p.render(Rect::new(0, 0, 80, 10), &mut new_buf);
        assert_eq!(format!("{:?}", old_buf), format!("{:?}", new_buf));
    }

    #[test]
    fn test_path_matches_neither_root() {
        let theme = Theme::default();
        let (old_p, new_p) = get_file_diff(OLD_DIR, NEW_DIR, "/nonexistent/path", 0, 10, &theme);

        use ratatui::buffer::Buffer;
        use ratatui::layout::Rect;
        use ratatui::widgets::Widget;

        let mut old_buf = Buffer::empty(Rect::new(0, 0, 80, 10));
        let mut new_buf = Buffer::empty(Rect::new(0, 0, 80, 10));
        old_p.render(Rect::new(0, 0, 80, 10), &mut old_buf);
        new_p.render(Rect::new(0, 0, 80, 10), &mut new_buf);
        // Both read attempts fail → both panes are empty
        assert_eq!(format!("{:?}", old_buf), format!("{:?}", new_buf));
    }

    #[test]
    fn test_identical_old_new_root() {
        let theme = Theme::default();
        let (old_p, new_p) = get_file_diff(
            OLD_DIR,
            OLD_DIR,
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
        old_p.render(Rect::new(0, 0, 80, 10), &mut old_buf);
        new_p.render(Rect::new(0, 0, 80, 10), &mut new_buf);
        assert_eq!(format!("{:?}", old_buf), format!("{:?}", new_buf));
    }

    #[test]
    fn test_scroll_offset_zero() {
        let theme = Theme::default();
        let (old_p, _new_p) = get_file_diff(
            OLD_DIR,
            NEW_DIR,
            "test_files/old/many_lines.txt",
            0,
            5,
            &theme,
        );

        use ratatui::buffer::Buffer;
        use ratatui::layout::Rect;
        use ratatui::widgets::Widget;

        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 5));
        old_p.render(Rect::new(0, 0, 80, 5), &mut buf);
        let rendered = format!("{:?}", buf);
        assert!(rendered.contains("001"), "should start at beginning");
    }

    #[test]
    fn test_scroll_offset_mid_file() {
        let theme = Theme::default();
        let (old_p, _new_p) = get_file_diff(
            OLD_DIR,
            NEW_DIR,
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
        assert!(rendered.contains("006"), "should start around line 006");
    }

    #[test]
    fn test_scroll_offset_beyond_file() {
        let theme = Theme::default();
        let (old_p, _new_p) = get_file_diff(
            OLD_DIR,
            NEW_DIR,
            "test_files/old/single_line.txt",
            100,
            5,
            &theme,
        );

        use ratatui::buffer::Buffer;
        use ratatui::layout::Rect;
        use ratatui::widgets::Widget;

        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 5));
        old_p.render(Rect::new(0, 0, 80, 5), &mut buf);
    }

    #[test]
    fn test_scroll_height_exceeds_file() {
        let theme = Theme::default();
        let (old_p, _new_p) = get_file_diff(
            OLD_DIR,
            NEW_DIR,
            "test_files/old/single_line.txt",
            0,
            500,
            &theme,
        );

        use ratatui::buffer::Buffer;
        use ratatui::layout::Rect;
        use ratatui::widgets::Widget;

        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 500));
        old_p.render(Rect::new(0, 0, 80, 500), &mut buf);
    }

    #[test]
    fn test_empty_file_identical() {
        let theme = Theme::default();
        let (old_p, new_p) =
            get_file_diff(OLD_DIR, NEW_DIR, "test_files/old/empty.txt", 0, 10, &theme);

        use ratatui::buffer::Buffer;
        use ratatui::layout::Rect;
        use ratatui::widgets::Widget;

        let mut old_buf = Buffer::empty(Rect::new(0, 0, 80, 10));
        let mut new_buf = Buffer::empty(Rect::new(0, 0, 80, 10));
        old_p.render(Rect::new(0, 0, 80, 10), &mut old_buf);
        new_p.render(Rect::new(0, 0, 80, 10), &mut new_buf);
        assert_eq!(format!("{:?}", old_buf), format!("{:?}", new_buf));
    }

    #[test]
    fn test_modified_file_shows_diff() {
        let theme = Theme::default();
        let (old_p, new_p) = get_file_diff(
            OLD_DIR,
            NEW_DIR,
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
        assert_ne!(format!("{:?}", old_buf), format!("{:?}", new_buf));
    }

    #[test]
    fn test_file_only_in_new_side() {
        let theme = Theme::default();
        let (old_p, _new_p) = get_file_diff(
            OLD_DIR,
            NEW_DIR,
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
    }

    #[test]
    fn test_file_only_in_old_side() {
        let theme = Theme::default();
        let (_old_p, new_p) = get_file_diff(
            OLD_DIR,
            NEW_DIR,
            "test_files/old/deleted_only_old.txt",
            0,
            10,
            &theme,
        );

        use ratatui::buffer::Buffer;
        use ratatui::layout::Rect;
        use ratatui::widgets::Widget;

        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 10));
        new_p.render(Rect::new(0, 0, 80, 10), &mut buf);
    }

    #[test]
    fn test_binary_file_does_not_panic() {
        let theme = Theme::default();
        let (_old_p, _new_p) =
            get_file_diff(OLD_DIR, NEW_DIR, "test_files/old/binary.bin", 0, 10, &theme);
    }

    #[test]
    fn test_unicode_filename() {
        let theme = Theme::default();
        let (_old_p, _new_p) =
            get_file_diff(OLD_DIR, NEW_DIR, "test_files/old/中文.txt", 0, 10, &theme);
    }

    #[test]
    fn test_spaces_in_filename() {
        let theme = Theme::default();
        let (_old_p, _new_p) = get_file_diff(
            OLD_DIR,
            NEW_DIR,
            "test_files/old/file with spaces.txt",
            0,
            10,
            &theme,
        );
    }

    #[test]
    fn test_rel_path_preserves_existing_slash() {
        let theme = Theme::default();
        let (old_p, new_p) = get_file_diff(
            "test_files/old",
            "test_files/new",
            "test_files/old/nested/deep/deeper/unchanged.txt",
            0,
            10,
            &theme,
        );

        use ratatui::buffer::Buffer;
        use ratatui::layout::Rect;
        use ratatui::widgets::Widget;

        let mut old_buf = Buffer::empty(Rect::new(0, 0, 80, 10));
        let mut new_buf = Buffer::empty(Rect::new(0, 0, 80, 10));
        old_p.render(Rect::new(0, 0, 80, 10), &mut old_buf);
        new_p.render(Rect::new(0, 0, 80, 10), &mut new_buf);
        assert_eq!(format!("{:?}", old_buf), format!("{:?}", new_buf));
    }

    #[test]
    fn test_whitespace_changes_produce_diff() {
        let theme = Theme::default();
        let (old_p, new_p) = get_file_diff(
            OLD_DIR,
            NEW_DIR,
            "test_files/old/whitespace_diff.txt",
            0,
            10,
            &theme,
        );

        use ratatui::buffer::Buffer;
        use ratatui::layout::Rect;
        use ratatui::widgets::Widget;

        let mut old_buf = Buffer::empty(Rect::new(0, 0, 80, 10));
        let mut new_buf = Buffer::empty(Rect::new(0, 0, 80, 10));
        old_p.render(Rect::new(0, 0, 80, 10), &mut old_buf);
        new_p.render(Rect::new(0, 0, 80, 10), &mut new_buf);
        assert_ne!(format!("{:?}", old_buf), format!("{:?}", new_buf));
    }

    #[test]
    fn test_theme_colors_applied_to_diff() {
        let theme = Theme {
            added: Color::Blue,
            removed: Color::Yellow,
            padding: Color::Cyan,
        };
        let (old_p, new_p) = get_file_diff(
            OLD_DIR,
            NEW_DIR,
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
        assert_ne!(format!("{:?}", old_buf), format!("{:?}", new_buf));
    }

    #[test]
    fn test_deeply_nested_unchanged() {
        let theme = Theme::default();
        let (old_p, new_p) = get_file_diff(
            OLD_DIR,
            NEW_DIR,
            "test_files/old/nested/deep/deeper/unchanged.txt",
            0,
            10,
            &theme,
        );

        use ratatui::buffer::Buffer;
        use ratatui::layout::Rect;
        use ratatui::widgets::Widget;

        let mut old_buf = Buffer::empty(Rect::new(0, 0, 80, 10));
        let mut new_buf = Buffer::empty(Rect::new(0, 0, 80, 10));
        old_p.render(Rect::new(0, 0, 80, 10), &mut old_buf);
        new_p.render(Rect::new(0, 0, 80, 10), &mut new_buf);
        assert_eq!(format!("{:?}", old_buf), format!("{:?}", new_buf));
    }

    #[test]
    fn test_many_hunks_produces_diff() {
        let theme = Theme::default();
        let (old_p, new_p) = get_file_diff(
            OLD_DIR,
            NEW_DIR,
            "test_files/old/many_hunks.txt",
            0,
            50,
            &theme,
        );

        use ratatui::buffer::Buffer;
        use ratatui::layout::Rect;
        use ratatui::widgets::Widget;

        let mut old_buf = Buffer::empty(Rect::new(0, 0, 80, 50));
        let mut new_buf = Buffer::empty(Rect::new(0, 0, 80, 50));
        old_p.render(Rect::new(0, 0, 80, 50), &mut old_buf);
        new_p.render(Rect::new(0, 0, 80, 50), &mut new_buf);
        assert_ne!(format!("{:?}", old_buf), format!("{:?}", new_buf));
    }

    #[test]
    fn test_unchanged_file_scrolls_old_pane() {
        let theme = Theme::default();
        let old_dir = tempfile::tempdir().unwrap();
        let new_dir = tempfile::tempdir().unwrap();
        let content: String = (0..40)
            .map(|i| format!("Line {:03}: scrolling test line\n", i + 1))
            .collect();
        std::fs::write(old_dir.path().join("same.txt"), &content).unwrap();
        std::fs::write(new_dir.path().join("same.txt"), &content).unwrap();

        let old_root = old_dir.path().to_str().unwrap();
        let new_root = new_dir.path().to_str().unwrap();
        let path = format!("{}/same.txt", old_root);

        let (old_p, _new_p) = get_file_diff(old_root, new_root, &path, 5, 10, &theme);

        use ratatui::buffer::Buffer;
        use ratatui::layout::Rect;
        use ratatui::widgets::Widget;

        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 10));
        old_p.render(Rect::new(0, 0, 80, 10), &mut buf);
        let rendered = format!("{:?}", buf);
        assert!(
            rendered.contains("Line 006"),
            "scrolled pane should start at line 006"
        );
        assert!(
            !rendered.contains("Line 001"),
            "scrolled pane should not show the first line"
        );
    }
}
