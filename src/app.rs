use crate::config::Theme;
use crate::state::TuiState;
use crate::ui;
use crate::walker::parallel_dir_load;
use clap::Parser;
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::time::Instant;

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

fn load_path(path: &PathBuf) -> (HashMap<String, Vec<String>>, String) {
    if path.is_file() {
        let root = path
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| ".".to_string());
        let file_name = path
            .file_name()
            .expect("Failed to get file name")
            .to_string_lossy()
            .to_string();
        let mut map = HashMap::new();
        map.insert(root.clone(), vec![file_name]);
        (map, root)
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

impl App {
    pub fn run(self) -> anyhow::Result<()> {
        let start = Instant::now();

        let (oldmap, old_root) = load_path(&self.old_dir);
        let (newmap, new_root) = load_path(&self.new_dir);

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

        let status = format!("TTT: {}ms; TAB: Toggle", start.elapsed().as_millis());
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
