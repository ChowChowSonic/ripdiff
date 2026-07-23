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

impl App {
    pub fn run(self) -> anyhow::Result<()> {
        let start = Instant::now();

        let oldmap = {
            let mut map = HashMap::new();
            map.extend(
                parallel_dir_load(&self.old_dir)
                    .lock()
                    .expect("Unable to lock old file set")
                    .drain(),
            );
            map
        };

        let newmap = {
            let mut map = HashMap::new();
            map.extend(
                parallel_dir_load(&self.new_dir)
                    .lock()
                    .expect("Unable to lock new file set")
                    .drain(),
            );
            map
        };

        log::info!("Read files in {:?}", start.elapsed());

        let old_root = self.old_dir.to_string_lossy().to_string();
        let new_root = self.new_dir.to_string_lossy().to_string();

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

        let status = format!("Files: {:?}", oldmap.len() + newmap.len());
        let theme = Theme::default();

        let mut state = TuiState::new(
            old_root,
            new_root,
            oldmap,
            newmap,
            folder_display,
            status,
            theme,
        );

        ratatui::run(|terminal| {
            ui::run(&mut state, terminal).expect("TUI error");
        });

        Ok(())
    }
}
