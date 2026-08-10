use crate::config::Theme;
use crate::state::TuiState;
use crate::ui;
use crate::walker::parallel_dir_load;
use clap::Parser;
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use tempfile::tempdir;

#[derive(Parser)]
#[command(
    name = "ripdiff",
    version,
    about = "Interactive terminal-based diff tool"
)]
pub struct App {
    pub old_dir: PathBuf,
    pub new_dir: PathBuf,
}

pub fn load_path(path: &PathBuf, temp_path: &Path) -> (HashMap<String, Vec<String>>, String) {
    if path.is_file() {
        fs::create_dir_all(temp_path).expect("Unable to create temp directory");
        let dest = temp_path.join("file.txt");
        fs::copy(path, &dest).expect("Failed to copy file to temp directory");
        let mut map = HashMap::new();
        map.insert(
            temp_path
                .to_str()
                .expect("Unable to retrieve temporary path")
                .to_string(),
            vec!["file.txt".to_string()],
        );
        (map, temp_path.to_string_lossy().to_string())
    } else {
        let root = path.to_string_lossy().to_string();
        let mut map = HashMap::new();
        map.extend(
            parallel_dir_load(path)
                .lock()
                .expect("Unable to lock file set")
                .drain(),
        );
        (map, root)
    }
}

pub fn build_status(elapsed: Duration) -> String {
    format!("Scan: {}ms | TAB: Toggle", elapsed.as_millis())
}

impl App {
    pub fn run(self) -> anyhow::Result<()> {
        let start = Instant::now();
        let temp_dir = tempdir()?;
        let new_dir = temp_dir.path().join("new");
        let old_dir = temp_dir.path().join("old");
        let (oldmap, old_root) = load_path(&self.old_dir, &old_dir);
        let (newmap, new_root) = load_path(&self.new_dir, &new_dir);

        log::info!("Read files in {:?}", start.elapsed());

        let mut folder_display: Vec<(String, String)> = oldmap
            .get(&old_root)
            .cloned()
            .unwrap_or_default()
            .iter()
            .map(|x| (old_root.clone(), x.clone()))
            .collect();

        let fd2: Vec<(String, String)> = newmap
            .get(&new_root)
            .cloned()
            .unwrap_or_default()
            .iter()
            .map(|x| (new_root.clone(), x.clone()))
            .collect();

        folder_display.extend(fd2);
        let mut seen = HashSet::new();
        folder_display.retain(|(_, f)| seen.insert(f.clone()));
        folder_display.par_sort_unstable();
        folder_display.reverse();

        let status = build_status(start.elapsed());
        let theme = Theme::default();

        let mut state = TuiState::new(
            old_root,
            new_root,
            oldmap,
            newmap,
            folder_display,
            status,
            theme,
            self.new_dir.is_file() && self.old_dir.is_file(),
        );

        ratatui::run(|terminal| {
            ui::run(&mut state, terminal).expect("TUI error");
        });

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_status_contains_elapsed_ms() {
        let status = build_status(Duration::from_millis(1234));
        assert!(status.contains("1234"), "status should show elapsed ms");
        assert!(status.contains("ms"), "status should include unit");
    }

    #[test]
    fn test_build_status_zero_time() {
        let status = build_status(Duration::ZERO);
        assert!(status.contains("0ms"), "zero elapsed should render as 0ms");
    }

    #[test]
    fn test_build_status_no_placeholder() {
        let status = build_status(Duration::from_millis(5));
        assert!(!status.contains("TTT"), "placeholder should be removed");
        assert!(status.contains("Scan:"), "status should describe the scan");
    }
}
