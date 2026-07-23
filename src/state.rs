use crate::config::Theme;
use ratatui::widgets::ListState;
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};

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
    pub theme: Theme,
    pub hide_sidebar: bool,
}

impl TuiState {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        old_root: String,
        new_root: String,
        old_files: HashMap<String, Vec<String>>,
        new_files: HashMap<String, Vec<String>>,
        file_display: Vec<(String, String)>,
        bottom_status: String,
        theme: Theme,
        hide_sidebar: bool,
    ) -> Self {
        let starting_file = if hide_sidebar {
            file_display
                .first()
                .map(|(dir, file)| format!("{dir}/{}", file.trim_start()))
        } else {
            None
        };
        Self {
            old_root,
            new_root,
            current_file: starting_file,
            old_files,
            new_files,
            file_display,
            bottom_status,
            file_name_offset: 0,
            file_scroll_offset: 0,
            state: ListState::default(),
            exit: false,
            open_files: Vec::new(),
            theme,
            hide_sidebar,
        }
    }

    pub fn open_file_or_dir(&mut self) {
        let selected = self.state.selected().unwrap_or(0);
        let tmpval = ("".to_string(), "".to_string());
        let current_dir = self.file_display.get(selected).unwrap_or(&tmpval);
        let mut full_path: String = current_dir.0.clone();
        full_path.push('/');
        full_path.push_str(current_dir.1.trim_start());
        let mut children = get_joined_paths(&full_path, &self.old_files, &self.new_files);
        let mut seen = HashSet::new();
        children.retain(|x| seen.insert(x.clone()));
        if !children.is_empty() {
            if self.open_files.contains(&full_path) {
                self.close_dir(&full_path, &children);
            } else {
                for x in &children {
                    let mut tmp_display = String::new();
                    let fp_temp =
                        full_path
                            .replacen(&self.old_root, "", 1)
                            .replacen(&self.new_root, "", 1);
                    for _y in fp_temp.match_indices("/") {
                        tmp_display.push_str("  ");
                    }
                    tmp_display.push_str(x);
                    self.file_display
                        .insert(selected + 1, (full_path.clone(), tmp_display));
                }
                self.open_files.push(full_path);
            }
            return;
        }
        self.current_file = Some(full_path);
    }

    fn close_dir(&mut self, path: &str, children: &[String]) {
        for x in children {
            let full_path = format!("{}/{}", path, x.trim_start());
            if self.open_files.contains(&full_path) {
                let child_dirs = get_joined_paths(&full_path, &self.old_files, &self.new_files);
                self.close_dir(&full_path, &child_dirs);
            }
        }
        self.file_display.retain(|(pth, file)| {
            let file_trimmed = file.trim();
            !(pth == path && children.iter().any(|x| x == file_trimmed))
        });
        let ind = self
            .open_files
            .iter()
            .position(|x| x == path)
            .expect("Failed to find path in open files");
        self.open_files.remove(ind);
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_state() -> TuiState {
        TuiState {
            old_root: "/root".to_string(),
            new_root: "/root".to_string(),
            current_file: None,
            old_files: HashMap::new(),
            new_files: HashMap::new(),
            file_display: Vec::new(),
            bottom_status: String::new(),
            file_name_offset: 0,
            file_scroll_offset: 0,
            state: ListState::default(),
            exit: false,
            open_files: Vec::new(),
            theme: Theme::default(),
            hide_sidebar: false,
        }
    }

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
    fn test_get_joined_paths_both_empty_returns_none() {
        let old = HashMap::new();
        let new = HashMap::new();
        let result = get_joined_paths("/dir", &old, &new);
        assert!(result.is_empty());
    }

    #[test]
    fn test_new_hide_sidebar_true_sets_current_file() {
        let state = TuiState::new(
            "/root".into(),
            "/root".into(),
            HashMap::new(),
            HashMap::new(),
            vec![("/root".into(), "file.txt".into())],
            "status".into(),
            Theme::default(),
            true,
        );
        assert_eq!(state.current_file, Some("/root/file.txt".to_string()));
    }

    #[test]
    fn test_new_hide_sidebar_false_no_current_file() {
        let state = TuiState::new(
            "/root".into(),
            "/root".into(),
            HashMap::new(),
            HashMap::new(),
            vec![("/root".into(), "file.txt".into())],
            "status".into(),
            Theme::default(),
            false,
        );
        assert_eq!(state.current_file, None);
    }

    #[test]
    fn test_new_hide_sidebar_true_empty_display() {
        let state = TuiState::new(
            "/root".into(),
            "/root".into(),
            HashMap::new(),
            HashMap::new(),
            vec![],
            "status".into(),
            Theme::default(),
            true,
        );
        assert_eq!(state.current_file, None);
    }

    #[test]
    fn test_new_default_fields() {
        let state = TuiState::new(
            "/a".into(),
            "/b".into(),
            HashMap::new(),
            HashMap::new(),
            vec![],
            "s".into(),
            Theme::default(),
            false,
        );
        assert_eq!(state.file_name_offset, 0);
        assert_eq!(state.file_scroll_offset, 0);
        assert!(!state.exit);
        assert!(state.open_files.is_empty());
        assert_eq!(state.old_root, "/a");
        assert_eq!(state.new_root, "/b");
    }

    #[test]
    fn test_close_dir_recursive_nesting() {
        let mut state = make_state();
        state.old_files = HashMap::from([
            ("/a".to_string(), vec!["b".to_string()]),
            ("/a/b".to_string(), vec!["c".to_string()]),
        ]);
        state.open_files = vec!["/a".to_string(), "/a/b".to_string(), "/a/b/c".to_string()];
        state.file_display = vec![
            ("/".to_string(), "a".to_string()),
            ("/a".to_string(), "  b".to_string()),
            ("/a/b".to_string(), "    c".to_string()),
        ];
        let children_of_a = vec!["b".to_string()];

        state.close_dir("/a", &children_of_a);

        assert!(!state.open_files.contains(&"/a".to_string()));
        assert!(!state.open_files.contains(&"/a/b".to_string()));
        assert!(!state.open_files.contains(&"/a/b/c".to_string()));
        // Root entry ("/", "a") survives since its parent is "/", not "/a"
        assert_eq!(state.file_display.len(), 1);
        assert_eq!(state.file_display[0].0, "/");
    }

    #[test]
    fn test_close_dir_partial_recursive_skip_not_open() {
        let mut state = make_state();
        state.old_files = HashMap::from([("/a".to_string(), vec!["b".to_string()])]);
        state.open_files = vec!["/a".to_string(), "/a/b".to_string()];
        state.file_display = vec![
            ("/".to_string(), "a".to_string()),
            ("/a".to_string(), "  b".to_string()),
            ("/a".to_string(), "  c".to_string()),
        ];
        let children_of_a = vec!["b".to_string(), "c".to_string()];

        state.close_dir("/a", &children_of_a);

        assert!(!state.open_files.contains(&"/a".to_string()));
        assert!(!state.open_files.contains(&"/a/b".to_string()));
        // Root entry "/" → "a" survives, children removed
        assert_eq!(state.file_display.len(), 1);
    }

    #[test]
    fn test_close_dir_no_children_noop() {
        let mut state = make_state();
        state.open_files = vec!["/root".to_string()];
        state.file_display = vec![("/root".to_string(), "file".to_string())];
        let children: Vec<String> = vec![];

        state.close_dir("/root", &children);

        assert!(state.open_files.is_empty());
        assert_eq!(state.file_display.len(), 1);
        assert_eq!(state.file_display[0].1, "file");
    }

    #[test]
    fn test_close_dir_does_not_affect_same_named_entries_from_other_paths() {
        let mut state = make_state();
        state.open_files = vec![
            "/reorg".to_string(),
            "/reorg/same_name".to_string(),
            "/same_name".to_string(),
            "/same_name/same_name".to_string(),
        ];
        state.file_display = vec![
            ("/".to_string(), "reorg".to_string()),
            ("/reorg".to_string(), "  same_name".to_string()),
            ("/".to_string(), "same_name".to_string()),
            ("/same_name".to_string(), "  same_name".to_string()),
        ];
        let children = vec!["same_name".to_string()];

        state.close_dir("/reorg", &children);

        assert!(!state.open_files.contains(&"/reorg".to_string()));
        assert!(!state.open_files.contains(&"/reorg/same_name".to_string()));
        assert!(state.open_files.contains(&"/same_name".to_string()));
        assert!(
            state
                .open_files
                .contains(&"/same_name/same_name".to_string())
        );
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
            state: ListState::default(),
            exit: false,
            open_files: Vec::new(),
            theme: Theme::default(),
            hide_sidebar: false,
        };
        state.state.select_first();

        state.open_file_or_dir();

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
            state: ListState::default(),
            exit: false,
            open_files: Vec::new(),
            theme: Theme::default(),
            hide_sidebar: false,
        };
        state.state.select_first();

        state.open_file_or_dir();

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
            state: ListState::default(),
            exit: false,
            open_files: vec!["/old/dir".to_string()],
            theme: Theme::default(),
            hide_sidebar: false,
        };
        state.state.select_first();

        state.open_file_or_dir();

        assert!(!state.open_files.contains(&"/old/dir".to_string()));
    }

    #[test]
    fn test_open_file_or_dir_no_selection_defaults_to_first() {
        let mut state = TuiState {
            old_root: "/root".to_string(),
            new_root: "/root".to_string(),
            current_file: None,
            old_files: HashMap::from([("/root".to_string(), vec!["a.txt".to_string()])]),
            new_files: HashMap::new(),
            file_display: vec![("/root".to_string(), "a.txt".to_string())],
            bottom_status: String::new(),
            file_name_offset: 0,
            file_scroll_offset: 0,
            state: ListState::default(),
            exit: false,
            open_files: Vec::new(),
            theme: Theme::default(),
            hide_sidebar: false,
        };
        // No select_first() called → selected() returns None

        state.open_file_or_dir();

        assert_eq!(state.current_file, Some("/root/a.txt".to_string()));
    }

    #[test]
    fn test_open_file_or_dir_empty_file_display() {
        let mut state = make_state();
        // No select called, empty display

        state.open_file_or_dir();

        // Should handle gracefully without panicking
        assert_eq!(state.current_file, Some("/".to_string()));
    }

    #[test]
    fn test_open_file_or_dir_children_from_both_sides_deduped() {
        let mut state = TuiState {
            old_root: "/root".to_string(),
            new_root: "/root".to_string(),
            current_file: None,
            old_files: HashMap::from([
                ("/root".to_string(), vec!["dir".to_string()]),
                (
                    "/root/dir".to_string(),
                    vec!["shared.txt".to_string(), "old_only.txt".to_string()],
                ),
            ]),
            new_files: HashMap::from([(
                "/root/dir".to_string(),
                vec!["shared.txt".to_string(), "new_only.txt".to_string()],
            )]),
            file_display: vec![("/root".to_string(), "dir".to_string())],
            bottom_status: String::new(),
            file_name_offset: 0,
            file_scroll_offset: 0,
            state: ListState::default(),
            exit: false,
            open_files: Vec::new(),
            theme: Theme::default(),
            hide_sidebar: false,
        };
        state.state.select_first();

        state.open_file_or_dir();

        assert!(state.open_files.contains(&"/root/dir".to_string()));
        // shared.txt should only appear once in file_display children
        let shared_entries: Vec<&(String, String)> = state
            .file_display
            .iter()
            .filter(|(_, f)| f.trim() == "shared.txt")
            .collect();
        assert_eq!(shared_entries.len(), 1, "shared.txt should be deduped");
    }

    #[test]
    fn test_open_file_or_dir_multi_level_open_close() {
        let mut state = TuiState {
            old_root: "/root".to_string(),
            new_root: "/root".to_string(),
            current_file: None,
            old_files: HashMap::from([
                ("/root".to_string(), vec!["a".to_string()]),
                ("/root/a".to_string(), vec!["b".to_string()]),
                ("/root/a/b".to_string(), vec!["c.txt".to_string()]),
            ]),
            new_files: HashMap::new(),
            file_display: vec![("/root".to_string(), "a".to_string())],
            bottom_status: String::new(),
            file_name_offset: 0,
            file_scroll_offset: 0,
            state: ListState::default(),
            exit: false,
            open_files: Vec::new(),
            theme: Theme::default(),
            hide_sidebar: false,
        };
        state.state.select_first();

        // open a → shows b
        state.open_file_or_dir();
        assert!(state.open_files.contains(&"/root/a".to_string()));

        state.state.select_next();

        // open b → shows c.txt
        state.open_file_or_dir();
        assert!(state.open_files.contains(&"/root/a/b".to_string()));

        state.state.select_previous();

        // close a → should recursively close a/b too
        state.open_file_or_dir();
        assert!(!state.open_files.contains(&"/root/a".to_string()));
        assert!(!state.open_files.contains(&"/root/a/b".to_string()));
    }

    #[test]
    fn test_open_file_or_dir_root_from_new_side() {
        let mut state = TuiState {
            old_root: "/old".to_string(),
            new_root: "/new".to_string(),
            current_file: None,
            old_files: HashMap::new(),
            new_files: HashMap::from([("/new".to_string(), vec!["file.txt".to_string()])]),
            file_display: vec![("/new".to_string(), "file.txt".to_string())],
            bottom_status: String::new(),
            file_name_offset: 0,
            file_scroll_offset: 0,
            state: ListState::default(),
            exit: false,
            open_files: Vec::new(),
            theme: Theme::default(),
            hide_sidebar: false,
        };
        state.state.select_first();

        state.open_file_or_dir();

        assert_eq!(state.current_file, Some("/new/file.txt".to_string()));
    }
}
