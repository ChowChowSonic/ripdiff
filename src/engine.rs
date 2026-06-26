use crate::tui::TuiState;
use diffy::create_patch;
use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
};
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::fs;

pub fn get_joined_paths(
    path: &str,
    old_files: &HashMap<String, Vec<String>>,
    new_files: &HashMap<String, Vec<String>>,
) -> Vec<String> {
    let x = old_files.get(path).cloned().unwrap_or_default();
    let y = new_files.get(path).cloned().unwrap_or_default();
    let mut result = x;
    result.extend(y);
    result.par_sort_unstable();
    result.reverse();
    result
}

pub fn close_dir(
    open_files: &mut Vec<String>,
    path: &str,
    children: &[String],
    old_files: &HashMap<String, Vec<String>>,
    new_files: &HashMap<String, Vec<String>>,
    file_display: &mut Vec<(String, String)>,
) {
    for x in children {
        let full_path = format!("{}/{}", path, x.trim_start());
        if open_files.contains(&full_path) {
            let child_dirs = get_joined_paths(&full_path, old_files, new_files);
            close_dir(
                open_files,
                &full_path,
                &child_dirs,
                old_files,
                new_files,
                file_display,
            );
        }
    }
    file_display.retain(|(pth, file)| {
        let file_trimmed = file.trim();
        !(pth == path && children.iter().any(|x| x == file_trimmed))
    });
    let ind = open_files
        .iter()
        .position(|x| x == path)
        .expect("Failed to find path in open files");
    open_files.remove(ind);
}

pub fn open_file_or_dir(state: &mut TuiState) {
    let selected = state.state.selected().unwrap_or(0);
    let tmpval = ("".to_string(), "".to_string());
    let current_dir = state.file_display.get(selected).unwrap_or(&tmpval);
    let mut full_path: String = current_dir.0.clone();
    full_path.push('/');
    full_path.push_str(current_dir.1.trim_start());
    let mut children = get_joined_paths(&full_path, &state.old_files, &state.new_files);
    let mut seen = HashSet::new();
    children.retain(|x| seen.insert(x.clone()));
    if !children.is_empty() {
        if state.open_files.contains(&full_path) {
            close_dir(
                &mut state.open_files,
                &full_path,
                &children,
                &state.old_files,
                &state.new_files,
                &mut state.file_display,
            );
        } else {
            for x in &children {
                let mut tmp_display = String::new();
                let fp_temp =
                    full_path
                        .replacen(&state.old_root, "", 1)
                        .replacen(&state.new_root, "", 1);
                for _y in fp_temp.match_indices("/") {
                    tmp_display.push_str("  ");
                }
                tmp_display.push_str(x);
                state
                    .file_display
                    .insert(selected + 1, (full_path.clone(), tmp_display));
            }
            state.open_files.push(full_path);
        }
        return;
    }
    state.current_file = Some(full_path);
}

pub fn get_file_diff(
    state: &TuiState,
    path: &str,
    height: usize,
) -> (Paragraph<'static>, Paragraph<'static>) {
    let mut rel_path: String = if path.starts_with(&state.old_root) {
        path[state.old_root.len()..].to_string()
    } else if path.starts_with(&state.new_root) {
        path[state.new_root.len()..].to_string()
    } else {
        path.to_string()
    };
    if !rel_path.starts_with('/') {
        rel_path.insert(0, '/');
    }
    let file1 = format!("{}{}", state.old_root, rel_path);
    let file2 = format!("{}{}", state.new_root, rel_path);
    let mut old_lines: Vec<Line> = Vec::new();
    let mut new_lines: Vec<Line> = Vec::new();
    let old_file_content = fs::read_to_string(&file1)
        .unwrap_or_else(|e| format!("Error reading file {}:\n{}", &file1, e));

    let new_file_content = if file1 != file2 {
        fs::read_to_string(&file2)
            .unwrap_or_else(|e| format!("Error reading file {}:\n{}", &file2, e))
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
                            Style::new().bg(Color::DarkGray),
                        )));
                        num_modded_lines -= 1;
                    }
                    while num_modded_lines < 0 {
                        new_lines.push(Line::from(Span::styled(
                            " ".to_string(),
                            Style::new().bg(Color::DarkGray),
                        )));
                        num_modded_lines += 1;
                    }
                    let line = Line::from(Span::raw(content.to_string()));
                    current_line_idx += 1;
                    old_lines.push(line.clone());
                    new_lines.push(line.clone());
                }
                diffy::Line::Delete(content) => {
                    let line = Line::from(Span::styled(content.to_string(), Color::Red));
                    current_line_idx += 1;
                    old_lines.push(line);
                    num_modded_lines -= 1;
                }
                diffy::Line::Insert(content) => {
                    let line = Line::from(Span::styled(content.to_string(), Color::Green));
                    new_lines.push(line);
                    num_modded_lines += 1;
                }
            }
        }

        while num_modded_lines > 0 {
            old_lines.push(Line::from(Span::styled(
                " ".to_string(),
                Style::new().bg(Color::DarkGray),
            )));
            num_modded_lines -= 1;
        }
        while num_modded_lines < 0 {
            new_lines.push(Line::from(Span::styled(
                " ".to_string(),
                Style::new().bg(Color::DarkGray),
            )));
            num_modded_lines += 1;
        }
    }
    let stop = state.file_scroll_offset + height;

    old_lines = old_lines
        [state.file_scroll_offset.clamp(0, old_lines.len())..stop.clamp(0, old_lines.len())]
        .to_vec();
    new_lines = new_lines
        [state.file_scroll_offset.clamp(0, new_lines.len())..stop.clamp(0, new_lines.len())]
        .to_vec();
    (Paragraph::new(old_lines), Paragraph::new(new_lines))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_joined_paths_empty() {
        let old = HashMap::new();
        let new = HashMap::new();
        let result = get_joined_paths("/nonexistent", &old, &new);
        assert!(result.is_empty());
    }

    #[test]
    fn test_get_joined_paths_merges_both_sides() {
        let mut old = HashMap::new();
        old.insert(
            "/dir".to_string(),
            vec!["a.txt".to_string(), "b.txt".to_string()],
        );
        let mut new = HashMap::new();
        new.insert(
            "/dir".to_string(),
            vec!["b.txt".to_string(), "c.txt".to_string()],
        );
        let result = get_joined_paths("/dir", &old, &new);
        // Function merges both sides, sorts descending, but does NOT dedup
        assert_eq!(result, vec!["c.txt", "b.txt", "b.txt", "a.txt"]);
    }

    #[test]
    fn test_get_joined_paths_old_only() {
        let mut old = HashMap::new();
        old.insert("/dir".to_string(), vec!["x.txt".to_string()]);
        let new = HashMap::new();
        let result = get_joined_paths("/dir", &old, &new);
        assert_eq!(result, vec!["x.txt"]);
    }

    #[test]
    fn test_get_joined_paths_new_only() {
        let old = HashMap::new();
        let mut new = HashMap::new();
        new.insert("/dir".to_string(), vec!["y.txt".to_string()]);
        let result = get_joined_paths("/dir", &old, &new);
        assert_eq!(result, vec!["y.txt"]);
    }

    #[test]
    fn test_close_dir_does_not_affect_same_named_entries_from_other_paths() {
        let mut open_files = vec![
            "/reorg".to_string(),
            "/reorg/same_name".to_string(),
            "/same_name".to_string(),
            "/same_name/same_name".to_string(),
        ];
        let mut file_display = vec![
            ("/".to_string(), "reorg".to_string()),
            ("/reorg".to_string(), "  same_name".to_string()),
            ("/".to_string(), "same_name".to_string()),
            ("/same_name".to_string(), "  same_name".to_string()),
        ];
        let old = HashMap::new();
        let new = HashMap::new();
        let children = vec!["same_name".to_string()];

        close_dir(
            &mut open_files,
            "/reorg",
            &children,
            &old,
            &new,
            &mut file_display,
        );

        // /reorg should be removed from open_files
        assert!(!open_files.contains(&"/reorg".to_string()));
        // /reorg/same_name should be removed (recursive close of child)
        assert!(!open_files.contains(&"/reorg/same_name".to_string()));
        // /same_name should still be there (different parent)
        assert!(open_files.contains(&"/same_name".to_string()));
        // /same_name/same_name should still be there
        assert!(open_files.contains(&"/same_name/same_name".to_string()));

        // file_display should keep entries whose parent is NOT /reorg
        let remaining_parents: Vec<&str> = file_display
            .iter()
            .map(|(p, _)| p.as_str())
            .collect();
        assert!(
            remaining_parents.contains(&"/"),
            "root entries should survive"
        );
        assert!(
            remaining_parents.contains(&"/same_name"),
            "other path entries should survive"
        );
        assert!(
            !remaining_parents.contains(&"/reorg"),
            "/reorg entries should be removed"
        );
    }

    #[test]
    fn test_close_dir_removes_only_own_children() {
        let mut open_files = vec![
            "/other".to_string(),
            "/other/same_name".to_string(),
            "/same_name".to_string(),
            "/same_name/same_name".to_string(),
        ];
        let mut file_display = vec![
            ("/root".to_string(), "other/same_name".to_string()),
            ("/root".to_string(), "other".to_string()),
            ("/root".to_string(), "same_name".to_string()),
            ("/root".to_string(), "same_name/same_name".to_string()),
        ];
        let old = HashMap::new();
        let new = HashMap::new();
        let children = vec![
            "/other/same_name".to_string(),
            "/same_name/same_name".to_string(),
        ];

        close_dir(
            &mut open_files,
            "/same_name/same_name",
            &children,
            &old,
            &new,
            &mut file_display,
        );

        assert!(!open_files.contains(&"/same_name/same_name".to_string()));
        assert!(open_files.contains(&"/same_name".to_string()));
        assert!(!file_display.is_empty());
    }

    #[test]
    fn test_close_dir_no_children_noop() {
        let mut open_files: Vec<String> = vec!["/root".to_string()];
        let mut file_display = vec![("/root".to_string(), "file".to_string())];
        let old = HashMap::new();
        let new = HashMap::new();
        let children: Vec<String> = vec![];

        close_dir(
            &mut open_files,
            "/root",
            &children,
            &old,
            &new,
            &mut file_display,
        );

        assert!(open_files.is_empty());
        assert_eq!(file_display.len(), 1);
        assert_eq!(file_display[0].1, "file");
    }

    #[test]
    fn test_open_file_or_dir_opens_directory() {
        let mut state = TuiState {
            old_root: "/root".to_string(),
            new_root: "/root".to_string(),
            current_file: None,
            old_files: HashMap::from([
                ("/root".to_string(), vec!["dir".to_string()]),
                (
                    "/root/dir".to_string(),
                    vec!["a.txt".to_string(), "b.txt".to_string()],
                ),
            ]),
            new_files: HashMap::new(),
            file_display: vec![("/root".to_string(), "dir".to_string())],
            bottom_status: String::new(),
            file_name_offset: 0,
            file_scroll_offset: 0,
            state: ratatui::widgets::ListState::default(),
            exit: false,
            open_files: Vec::new(),
        };
        state.state.select_first();

        open_file_or_dir(&mut state);

        assert!(state.open_files.contains(&"/root/dir".to_string()));
        assert!(state.file_display.len() >= 3);
        assert!(state.file_display.iter().any(|(p, _)| p == "/root/dir"));
    }

    #[test]
    fn test_open_file_or_dir_opens_file_when_no_children() {
        let mut state = TuiState {
            old_root: "/root".to_string(),
            new_root: "/root".to_string(),
            current_file: None,
            old_files: HashMap::from([("/root".to_string(), vec!["file.txt".to_string()])]),
            new_files: HashMap::new(),
            file_display: vec![("/root".to_string(), "file.txt".to_string())],
            bottom_status: String::new(),
            file_name_offset: 0,
            file_scroll_offset: 0,
            state: ratatui::widgets::ListState::default(),
            exit: false,
            open_files: Vec::new(),
        };
        state.state.select_first();

        open_file_or_dir(&mut state);

        assert_eq!(state.current_file, Some("/root/file.txt".to_string()));
    }

    #[test]
    fn test_open_file_or_dir_closes_opened_directory() {
        let mut state = TuiState {
            old_root: "/old".to_string(),
            new_root: "/new".to_string(),
            current_file: None,
            old_files: HashMap::from([
                ("/old".to_string(), vec!["dir".to_string()]),
                ("/old/dir".to_string(), vec!["file.txt".to_string()]),
            ]),
            new_files: HashMap::new(),
            file_display: vec![
                ("/old".to_string(), "dir".to_string()),
                ("/old/dir".to_string(), "  file.txt".to_string()),
            ],
            bottom_status: String::new(),
            file_name_offset: 0,
            file_scroll_offset: 0,
            state: ratatui::widgets::ListState::default(),
            exit: false,
            open_files: vec!["/old/dir".to_string()],
        };
        state.state.select_first();

        open_file_or_dir(&mut state);

        // close_dir should remove /old/dir from open_files and
        // remove the indented child entry from file_display
        assert!(!state.open_files.contains(&"/old/dir".to_string()));
    }

    #[test]
    fn test_get_file_diff_unchanged_file() {
        let state = TuiState {
            old_root: "test_files/old".to_string(),
            new_root: "test_files/new".to_string(),
            current_file: None,
            old_files: HashMap::new(),
            new_files: HashMap::new(),
            file_display: Vec::new(),
            bottom_status: String::new(),
            file_name_offset: 0,
            file_scroll_offset: 0,
            state: ratatui::widgets::ListState::default(),
            exit: false,
            open_files: Vec::new(),
        };

        // Should not panic for an unchanged file that exists in both directories
        let (_old_p, _new_p) = get_file_diff(&state, "test_files/old/unchanged.txt", 10);

        use ratatui::buffer::Buffer;
        use ratatui::layout::Rect;
        use ratatui::widgets::Widget;

        let mut old_buf = Buffer::empty(Rect::new(0, 0, 80, 10));
        let mut new_buf = Buffer::empty(Rect::new(0, 0, 80, 10));
        _old_p.render(Rect::new(0, 0, 80, 10), &mut old_buf);
        _new_p.render(Rect::new(0, 0, 80, 10), &mut new_buf);
        // Both panes should render identical content for an unchanged file
        assert_eq!(format!("{:?}", old_buf), format!("{:?}", new_buf));
    }

    #[test]
    fn test_get_file_diff_missing_file_old_side() {
        let state = TuiState {
            old_root: "test_files/old".to_string(),
            new_root: "test_files/new".to_string(),
            current_file: None,
            old_files: HashMap::new(),
            new_files: HashMap::new(),
            file_display: Vec::new(),
            bottom_status: String::new(),
            file_name_offset: 0,
            file_scroll_offset: 0,
            state: ratatui::widgets::ListState::default(),
            exit: false,
            open_files: Vec::new(),
        };

        let (old_p, _new_p) = get_file_diff(&state, "test_files/old/added_only_new.txt", 10);

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
        let state = TuiState {
            old_root: "test_files/old".to_string(),
            new_root: "test_files/new".to_string(),
            current_file: None,
            old_files: HashMap::new(),
            new_files: HashMap::new(),
            file_display: Vec::new(),
            bottom_status: String::new(),
            file_name_offset: 0,
            file_scroll_offset: 5,
            state: ratatui::widgets::ListState::default(),
            exit: false,
            open_files: Vec::new(),
        };

        let (old_p, _new_p) = get_file_diff(&state, "test_files/old/many_lines.txt", 10);

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
        let state = TuiState {
            old_root: "test_files/old".to_string(),
            new_root: "test_files/new".to_string(),
            current_file: None,
            old_files: HashMap::new(),
            new_files: HashMap::new(),
            file_display: Vec::new(),
            bottom_status: String::new(),
            file_name_offset: 0,
            file_scroll_offset: 0,
            state: ratatui::widgets::ListState::default(),
            exit: false,
            open_files: Vec::new(),
        };

        let (old_p, _new_p) = get_file_diff(&state, "test_files/old/binary.bin", 10);

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
        let state = TuiState {
            old_root: "test_files/old".to_string(),
            new_root: "test_files/new".to_string(),
            current_file: None,
            old_files: HashMap::new(),
            new_files: HashMap::new(),
            file_display: Vec::new(),
            bottom_status: String::new(),
            file_name_offset: 0,
            file_scroll_offset: 0,
            state: ratatui::widgets::ListState::default(),
            exit: false,
            open_files: Vec::new(),
        };

        let (old_p, new_p) = get_file_diff(&state, "test_files/old/modified.txt", 20);

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
