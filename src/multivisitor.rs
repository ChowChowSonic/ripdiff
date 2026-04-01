use ignore::{DirEntry, ParallelVisitor, ParallelVisitorBuilder, WalkBuilder, WalkState};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{Arc, LazyLock, Mutex};

// 1. The Visitor: Holds thread-local data
pub struct MultiVisitor {
    local_files: HashMap<String, Vec<String>>,
    main_accumulator: Arc<Mutex<HashMap<String, Vec<String>>>>,
}

impl ParallelVisitor for MultiVisitor {
    fn visit(&mut self, entry: Result<ignore::DirEntry, ignore::Error>) -> WalkState {
        if let Ok(entry) = entry
            && let Ok(children) = entry.path().read_dir()
        {
            self.local_files.insert(
                entry.path().to_str().unwrap_or(&"").to_string(),
                children
                    .into_iter()
                    .filter_map(|x| x.ok())
                    .map(|x| {
                        x.file_name()
                            .to_str()
                            .expect("Failed to unwrap path")
                            .to_string()
                    })
                    .collect::<Vec<String>>(),
            );
        }
        WalkState::Continue
    }
}

// 2. The Drop Implementation: Merges data when the thread finishes
impl Drop for MultiVisitor {
    fn drop(&mut self) {
        let mut guard = self.main_accumulator.lock().unwrap();
        // Use drain() to move items out of the local vector efficiently
        guard.extend(self.local_files.drain());
        // Lock is released here as guard goes out of scope
    }
}

// 3. The Builder: Creates a new Visitor for every thread
pub struct MyVisitorBuilder {
    pub main_accumulator: Arc<Mutex<HashMap<String, Vec<String>>>>,
}

impl<'s> ParallelVisitorBuilder<'s> for MyVisitorBuilder {
    fn build(&mut self) -> Box<dyn ParallelVisitor + 's> {
        Box::new(MultiVisitor {
            local_files: HashMap::new(), // Pre-allocate to reduce resizing
            main_accumulator: Arc::clone(&self.main_accumulator),
        })
    }
}
