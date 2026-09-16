use crate::config::Theme;
use diffy::create_patch;
use ratatui::{
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
};
use std::collections::HashMap;
use std::fs;

#[derive(Clone)]
pub enum OwnedLineType {
    Context,
    Delete,
    Insert,
}

#[derive(Clone)]
pub struct OwnedHunk {
    pub old_start: usize,
    pub lines: Vec<(OwnedLineType, String)>,
}

pub fn change_region_offsets(hunks: &[OwnedHunk]) -> Vec<usize> {
    let mut offsets = Vec::new();
    let mut row = 0usize;
    let mut pending: i64 = 0;
    let mut current_line_idx = 0usize;
    let mut in_region = false;

    for hunk in hunks {
        let start_of_hunk = hunk.old_start.saturating_sub(1);
        row += start_of_hunk.saturating_sub(current_line_idx);
        current_line_idx = current_line_idx.max(start_of_hunk);

        for (line_type, _) in &hunk.lines {
            match line_type {
                OwnedLineType::Context => {
                    if pending > 0 {
                        row += pending as usize;
                    }
                    pending = 0;
                    row += 1;
                    current_line_idx += 1;
                    in_region = false;
                }
                OwnedLineType::Delete => {
                    if !in_region {
                        offsets.push(row);
                        in_region = true;
                    }
                    row += 1;
                    current_line_idx += 1;
                    pending -= 1;
                }
                OwnedLineType::Insert => {
                    if !in_region {
                        offsets.push(row);
                        in_region = true;
                    }
                    pending += 1;
                }
            }
        }
        if pending > 0 {
            row += pending as usize;
        }
        pending = 0;
        in_region = false;
    }

    offsets
}

pub struct DiffCacheEntry {
    pub old_content: String,
    pub hunks: Vec<OwnedHunk>,
    pub change_region_offsets: Vec<usize>,
}

fn hunks_from_patch(old_content: &str, new_content: &str) -> Vec<OwnedHunk> {
    let patch = create_patch(old_content, new_content);
    patch
        .hunks()
        .iter()
        .map(|h| OwnedHunk {
            old_start: h.old_range().start(),
            lines: h
                .lines()
                .iter()
                .map(|l| match l {
                    diffy::Line::Context(c) => (OwnedLineType::Context, c.to_string()),
                    diffy::Line::Delete(c) => (OwnedLineType::Delete, c.to_string()),
                    diffy::Line::Insert(c) => (OwnedLineType::Insert, c.to_string()),
                })
                .collect(),
        })
        .collect()
}

fn compute_styled_lines(
    old_content: &str,
    hunks: &[OwnedHunk],
    scroll_offset: usize,
    height: usize,
    theme: &Theme,
) -> (Vec<Line<'static>>, Vec<Line<'static>>) {
    let mut old_lines: Vec<Line> = Vec::new();
    let mut new_lines: Vec<Line> = Vec::new();
    let stop = scroll_offset + height;

    if hunks.is_empty() {
        old_lines = old_content
            .split("\n")
            .map(|x| Line::from(Span::styled(x.to_string(), Style::new())))
            .collect();
        old_lines = old_lines
            [scroll_offset.clamp(0, old_lines.len())..stop.clamp(0, old_lines.len())]
            .to_vec();
        let new_lines_full: Vec<Line> = old_content
            .split("\n")
            .map(|x| Line::from(Span::styled(x.to_string(), Style::new())))
            .collect();
        new_lines = new_lines_full
            [scroll_offset.clamp(0, new_lines_full.len())..stop.clamp(0, new_lines_full.len())]
            .to_vec();
        return (old_lines, new_lines);
    }

    let lines: Vec<_> = old_content.split('\n').collect();
    let mut current_line_idx = 0;
    for hunk in hunks {
        let start_of_hunk = hunk.old_start.saturating_sub(1);
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
        for (line_type, content) in &hunk.lines {
            match line_type {
                OwnedLineType::Context => {
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
                    let line = Line::from(Span::raw(content.clone()));
                    current_line_idx += 1;
                    old_lines.push(line.clone());
                    new_lines.push(line.clone());
                }
                OwnedLineType::Delete => {
                    let line = Line::from(Span::styled(content.clone(), theme.removed));
                    current_line_idx += 1;
                    old_lines.push(line);
                    num_modded_lines -= 1;
                }
                OwnedLineType::Insert => {
                    let line = Line::from(Span::styled(content.clone(), theme.added));
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
    (old_lines, new_lines)
}

pub fn get_file_diff(
    old_root: &str,
    new_root: &str,
    path: &str,
    scroll_offset: usize,
    height: usize,
    theme: &Theme,
    cache: &mut HashMap<String, DiffCacheEntry>,
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

    let (old_content, hunks) = if let Some(entry) = cache.get(path) {
        (entry.old_content.clone(), entry.hunks.clone())
    } else {
        let old_file_content = fs::read_to_string(&file1).unwrap_or_else(|_| "".to_string());
        let new_file_content = if file1 != file2 {
            fs::read_to_string(&file2).unwrap_or_else(|_| "".to_string())
        } else {
            old_file_content.clone()
        };
        let hunks = hunks_from_patch(&old_file_content, &new_file_content);
        cache.insert(
            path.to_string(),
            DiffCacheEntry {
                old_content: old_file_content.clone(),
                hunks: hunks.clone(),
                change_region_offsets: change_region_offsets(&hunks),
            },
        );
        (old_file_content, hunks)
    };

    let (old_lines, new_lines) =
        compute_styled_lines(&old_content, &hunks, scroll_offset, height, theme);
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
            &mut HashMap::new(),
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
            &mut HashMap::new(),
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
        let (old_p, new_p) = get_file_diff(
            OLD_DIR,
            NEW_DIR,
            "/nonexistent/path",
            0,
            10,
            &theme,
            &mut HashMap::new(),
        );

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
            &mut HashMap::new(),
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
            &mut HashMap::new(),
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
            &mut HashMap::new(),
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
            &mut HashMap::new(),
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
            &mut HashMap::new(),
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
        let (old_p, new_p) = get_file_diff(
            OLD_DIR,
            NEW_DIR,
            "test_files/old/empty.txt",
            0,
            10,
            &theme,
            &mut HashMap::new(),
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
    fn test_modified_file_shows_diff() {
        let theme = Theme::default();
        let (old_p, new_p) = get_file_diff(
            OLD_DIR,
            NEW_DIR,
            "test_files/old/modified.txt",
            0,
            20,
            &theme,
            &mut HashMap::new(),
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
            &mut HashMap::new(),
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
            &mut HashMap::new(),
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
        let (_old_p, _new_p) = get_file_diff(
            OLD_DIR,
            NEW_DIR,
            "test_files/old/binary.bin",
            0,
            10,
            &theme,
            &mut HashMap::new(),
        );
    }

    #[test]
    fn test_unicode_filename() {
        let theme = Theme::default();
        let (_old_p, _new_p) = get_file_diff(
            OLD_DIR,
            NEW_DIR,
            "test_files/old/中文.txt",
            0,
            10,
            &theme,
            &mut HashMap::new(),
        );
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
            &mut HashMap::new(),
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
            &mut HashMap::new(),
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
            &mut HashMap::new(),
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
            &mut HashMap::new(),
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
            &mut HashMap::new(),
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
            &mut HashMap::new(),
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
        let mut tmp_cache: HashMap<String, DiffCacheEntry> = HashMap::new();
        let (old_p, _new_p) =
            get_file_diff(old_root, new_root, &path, 5, 10, &theme, &mut tmp_cache);

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

    fn first_row_text(buf: &ratatui::buffer::Buffer) -> String {
        (0..buf.area.width)
            .map(|x| {
                buf.cell((x, 0))
                    .and_then(|c| c.symbol().chars().next())
                    .unwrap_or(' ')
            })
            .collect()
    }

    #[test]
    fn test_jump_offset_puts_change_on_first_line_fixture() {
        let theme = Theme::default();
        let path = "test_files/old/modified.txt";
        let mut cache: HashMap<String, DiffCacheEntry> = HashMap::new();
        let (_, _) = get_file_diff(OLD_DIR, NEW_DIR, path, 0, 10, &theme, &mut cache);

        let entry = cache.get(path).unwrap();
        let target = change_region_offsets(&entry.hunks)[0];
        assert_eq!(target, 1, "leading context is line 1, change at line 2");

        use ratatui::buffer::Buffer;
        use ratatui::layout::Rect;
        use ratatui::widgets::Widget;

        let (old_p, new_p) = get_file_diff(OLD_DIR, NEW_DIR, path, target, 10, &theme, &mut cache);
        let mut old_buf = Buffer::empty(Rect::new(0, 0, 80, 10));
        old_p.render(Rect::new(0, 0, 80, 10), &mut old_buf);
        assert!(
            first_row_text(&old_buf).starts_with("Line 2: This line will be modified"),
            "old pane first row should be the deleted line, got: {}",
            first_row_text(&old_buf)
        );

        let mut new_buf = Buffer::empty(Rect::new(0, 0, 80, 10));
        new_p.render(Rect::new(0, 0, 80, 10), &mut new_buf);
        assert!(
            first_row_text(&new_buf).starts_with("Line 2: THIS LINE HAS BEEN MODIFIED"),
            "new pane first row should be the inserted line, got: {}",
            first_row_text(&new_buf)
        );
    }

    #[test]
    fn test_jump_offset_puts_change_on_first_line_three_context() {
        let theme = Theme::default();
        let old_dir = tempfile::tempdir().unwrap();
        let new_dir = tempfile::tempdir().unwrap();
        let old_content: String = (0..10).map(|i| format!("Line {:03}\n", i + 1)).collect();
        let new_content = old_content.replacen("Line 005\n", "Line 005 CHANGED\n", 1);
        std::fs::write(old_dir.path().join("f.txt"), &old_content).unwrap();
        std::fs::write(new_dir.path().join("f.txt"), &new_content).unwrap();

        let old_root = old_dir.path().to_str().unwrap();
        let new_root = new_dir.path().to_str().unwrap();
        let path = format!("{}/f.txt", old_root);
        let mut cache: HashMap<String, DiffCacheEntry> = HashMap::new();
        let (_, _) = get_file_diff(old_root, new_root, &path, 0, 10, &theme, &mut cache);

        let entry = cache.get(&path).unwrap();
        let target = change_region_offsets(&entry.hunks)[0];
        assert_eq!(target, 4, "change is 0-based line 4 (line 005)");

        use ratatui::buffer::Buffer;
        use ratatui::layout::Rect;
        use ratatui::widgets::Widget;

        let (old_p, _new_p) =
            get_file_diff(old_root, new_root, &path, target, 10, &theme, &mut cache);
        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 10));
        old_p.render(Rect::new(0, 0, 80, 10), &mut buf);
        assert!(
            first_row_text(&buf).starts_with("Line 005"),
            "old pane first row should be the changed line, got: {}",
            first_row_text(&buf)
        );
    }

    #[test]
    fn test_change_region_offsets_merged_hunk_multiple_regions() {
        let hunks = vec![OwnedHunk {
            old_start: 1,
            lines: vec![
                (OwnedLineType::Context, "l1".into()),
                (OwnedLineType::Delete, "l2".into()),
                (OwnedLineType::Insert, "l2'".into()),
                (OwnedLineType::Context, "l3".into()),
                (OwnedLineType::Context, "l4".into()),
                (OwnedLineType::Delete, "l5".into()),
                (OwnedLineType::Insert, "l5'".into()),
                (OwnedLineType::Context, "l6".into()),
                (OwnedLineType::Insert, "a".into()),
                (OwnedLineType::Insert, "b".into()),
            ],
        }];
        assert_eq!(change_region_offsets(&hunks), vec![1, 4, 6]);
    }

    #[test]
    fn test_change_region_offsets_insert_padding_shifts_later_region() {
        // Old: l1..l45. One change replaces l2 with two lines (an insert),
        // another change hits l40. The insert adds two padding rows on the old
        // side, so l40's change (raw 0-based index 39) lands at display row 41.
        let hunks = vec![
            OwnedHunk {
                old_start: 1,
                lines: vec![
                    (OwnedLineType::Context, "l1".into()),
                    (OwnedLineType::Delete, "l2".into()),
                    (OwnedLineType::Insert, "l2'".into()),
                    (OwnedLineType::Insert, "extra".into()),
                ],
            },
            OwnedHunk {
                old_start: 37,
                lines: vec![
                    (OwnedLineType::Context, "l37".into()),
                    (OwnedLineType::Context, "l38".into()),
                    (OwnedLineType::Context, "l39".into()),
                    (OwnedLineType::Delete, "l40".into()),
                ],
            },
        ];
        assert_eq!(change_region_offsets(&hunks), vec![1, 40]);
    }

    #[test]
    fn test_many_hunks_fixture_visits_all_regions() {
        let theme = Theme::default();
        let path = "test_files/old/many_hunks.txt";
        let mut cache: HashMap<String, DiffCacheEntry> = HashMap::new();
        let (_, _) = get_file_diff(OLD_DIR, NEW_DIR, path, 0, 50, &theme, &mut cache);

        let entry = cache.get(path).unwrap();
        let offsets = entry.change_region_offsets.clone();
        assert!(
            offsets.len() >= 2,
            "the many-hunks fixture should split into multiple change regions, got {offsets:?}"
        );

        let mut scroll = 0;
        for expected in &offsets {
            let target = offsets.iter().copied().find(|o| *o > scroll);
            assert_eq!(target, Some(*expected));
            scroll = target.unwrap();
        }
        assert!(
            offsets.iter().all(|o| *o <= scroll),
            "all regions should be reachable by pressing '/' (last scroll {scroll}, offsets {offsets:?})"
        );
    }

    #[test]
    fn test_later_region_after_insert_puts_change_on_first_line() {
        // Old has an edit at line 3 (accompanied by an insert) and a later
        // edit at line 15, far enough apart that diffy splits them into two
        // hunks. The insert's padding row must not offset the later region.
        let theme = Theme::default();
        let old_dir = tempfile::tempdir().unwrap();
        let new_dir = tempfile::tempdir().unwrap();
        let old_content: String = (0..20).map(|i| format!("Line {:03}\n", i + 1)).collect();
        let new_content = format!(
            "Line 001\nLine 002\nLine 003 CHANGED\nINSERTED\nLine 004\nLine 005\n\
             Line 006\nLine 007\nLine 008\nLine 009\nLine 010\nLine 011\nLine 012\n\
             Line 013\nLine 014\nLine 015 CHANGED\nLine 016\nLine 017\nLine 018\n\
             Line 019\nLine 020\n"
        );
        std::fs::write(old_dir.path().join("f.txt"), &old_content).unwrap();
        std::fs::write(new_dir.path().join("f.txt"), &new_content).unwrap();

        let old_root = old_dir.path().to_str().unwrap();
        let new_root = new_dir.path().to_str().unwrap();
        let path = format!("{}/f.txt", old_root);
        let mut cache: HashMap<String, DiffCacheEntry> = HashMap::new();
        let (_, _) = get_file_diff(old_root, new_root, &path, 0, 20, &theme, &mut cache);
        let entry = cache.get(&path).unwrap();
        let offsets = entry.change_region_offsets.clone();
        assert_eq!(
            offsets.len(),
            2,
            "expected two change regions, got {offsets:?}"
        );

        use ratatui::buffer::Buffer;
        use ratatui::layout::Rect;
        use ratatui::widgets::Widget;

        let (old_p, _new_p) = get_file_diff(
            old_root, new_root, &path, offsets[1], 20, &theme, &mut cache,
        );
        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 20));
        old_p.render(Rect::new(0, 0, 80, 20), &mut buf);
        assert!(
            first_row_text(&buf).starts_with("Line 015"),
            "later change should render on the first row at offset {}, got: {}",
            offsets[1],
            first_row_text(&buf)
        );
    }
}
