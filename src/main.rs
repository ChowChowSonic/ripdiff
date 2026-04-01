use ratatui::widgets::ListState;
use rayon::prelude::*;
mod multivisitor;
mod tui;
use crate::multivisitor::MyVisitorBuilder;
use crate::tui::TuiState;
//use diffy::{PatchFormatter, create_patch};
use ignore::WalkBuilder;
use std::collections::HashSet;
use std::env;
use std::thread::available_parallelism;
use std::time::Instant;
use std::{collections::HashMap, path::PathBuf};
use std::{
    ops::Index,
    sync::{Arc, Mutex},
};
fn parallel_dir_load(dir: &PathBuf) -> Arc<Mutex<HashMap<String, Vec<String>>>> {
    let files: Arc<Mutex<HashMap<String, Vec<String>>>> = Arc::new(Mutex::new(HashMap::new()));
    let walker = WalkBuilder::new(dir)
        .standard_filters(false)
        .threads(available_parallelism().unwrap().get())
        .build_parallel();
    let mut builder = MyVisitorBuilder {
        main_accumulator: Arc::clone(&files),
    };
    walker.visit(&mut builder);
    files
}
fn main() -> Result<(), std::io::Error> {
    simple_logger::init().expect("Failed to initialize logger");
    let start = Instant::now();
    log::set_max_level(log::LevelFilter::Info);
    let args: Vec<String> = env::args().collect();
    match args.len() {
        0 => {
            println!("Please pass an old and new folder to diff between");
            return Ok(());
        }
        1 => {
            println!("Please pass an old and new folder to diff between");
            return Ok(());
        }
        2 => {
            println!("Please pass an old and new folder to diff between");
            return Ok(());
        }
        _ => {} //TODO: Fix integer underflow
    }

    let olddir = PathBuf::from(&args[1]);
    let mut oldmap: HashMap<String, Vec<String>> = HashMap::new();
    oldmap.extend(
        parallel_dir_load(&olddir)
            .lock()
            .expect("Unable to lock old file set")
            .drain(),
    );
    let t1 = start.elapsed();
    //let old_trie: Trie<u8> = init_btree_from(&args[1], oldmap.to_vec());
    let t2 = start.elapsed();

    let newdir = PathBuf::from(&args[2]);
    let mut newmap: HashMap<String, Vec<String>> = HashMap::new();
    newmap.extend(
        parallel_dir_load(&newdir)
            .lock()
            .expect("Unable to lock New file set")
            .drain(),
    );
    let t3 = start.elapsed();
    let t4 = start.elapsed();
    let t5 = start.elapsed();
    let t6 = start.elapsed();
    let t7 = start.elapsed();
    let t8 = start.elapsed();
    //let index = make_index(&total_folders);
    let t9 = start.elapsed();

    //log::info!("dirs under /checkup-db-100");
    //list_files("/checkup-db-1000".to_string(), &new_trie);
    //log::info!("dirs under /");
    //list_files("".to_string(), &new_trie);
    //log::info!("all dirs");
    /*
    for x in oldmap.iter() {
        log::info!("{:?}", &x);
    }
    // */
    log::info!("Read old files in {:?} ", t1);
    //log::info!("Initialized old file btree in {:?} ", t2 - t1);
    log::info!("Read new files in {:?} ", t3 - t2);
    //log::info!("Initialized new btree in {:?} ", t4 - t3);
    log::info!("Merged contents of old and new in {:?}", t4 - t3);
    log::info!("String conversion completed in {:?}", t5 - t4);
    log::info!("Directory sorting executed in {:?}", t6 - t5);
    log::info!("Built btree from files in {:?}", t7 - t6);
    log::info!("Retrieving root dirs comnpleted in {:?}", t8 - t7);
    ratatui::run(|terminal| {
        let size = terminal.size().expect("Unable to get terminal size");
        let mut fd1 = oldmap.index(&args[1]).clone();
        let mut folder_display: Vec<(String, String)> = fd1
            .iter()
            .map(|x| (args[1].to_string(), x.to_string()))
            .collect();
        let fd2: Vec<(String, String)> = newmap
            .index(&args[2])
            .clone()
            .iter()
            .map(|x| (args[2].to_string(), x.to_string()))
            .collect();
        folder_display.extend(fd2);
        folder_display = folder_display
            .into_par_iter()
            .collect::<HashSet<(String, String)>>()
            .into_par_iter()
            .collect();
        let status = format!(
            "TTT: {:?}; Files: {:?}",
            start.elapsed(),
            oldmap.len() + newmap.len()
        );
        let mut state = TuiState {
            old_root: args[1].to_string(),
            new_root: args[2].to_string(),
            current_file: None,
            old_files: oldmap,
            new_files: newmap,
            bottom_status: status,
            file_display: folder_display,
            terminal_size: size,
            file_name_offset: 0,
            file_scroll_offset: 0,
            state: ListState::default(),
            exit: false,
            open_files: Vec::new(),
        };

        state.run(terminal).expect("Failed to start TUI");
    });
    Ok(())
}
