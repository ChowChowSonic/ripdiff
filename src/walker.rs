use ignore::{ParallelVisitor, ParallelVisitorBuilder, WalkBuilder, WalkState};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread::available_parallelism;

pub struct MultiVisitor {
    local_files: HashMap<String, Vec<String>>,
    main_accumulator: Arc<Mutex<HashMap<String, Vec<String>>>>,
}

impl ParallelVisitor for MultiVisitor {
    fn visit(&mut self, entry: Result<ignore::DirEntry, ignore::Error>) -> WalkState {
        if let Ok(entry) = entry
            && let Ok(children) = entry.path().read_dir()
        {
            let childs = children
                .into_iter()
                .filter_map(|x| x.ok())
                .map(|x| {
                    x.file_name()
                        .to_str()
                        .expect("Failed to unwrap path")
                        .to_string()
                })
                .collect::<Vec<String>>();
            self.local_files
                .insert(entry.path().to_str().unwrap_or("").to_string(), childs);
        }
        WalkState::Continue
    }
}

impl Drop for MultiVisitor {
    fn drop(&mut self) {
        let mut guard = self.main_accumulator.lock().unwrap();
        guard.extend(self.local_files.drain());
    }
}

pub struct MyVisitorBuilder {
    pub main_accumulator: Arc<Mutex<HashMap<String, Vec<String>>>>,
}

impl<'s> ParallelVisitorBuilder<'s> for MyVisitorBuilder {
    fn build(&mut self) -> Box<dyn ParallelVisitor + 's> {
        Box::new(MultiVisitor {
            local_files: HashMap::new(),
            main_accumulator: Arc::clone(&self.main_accumulator),
        })
    }
}

pub fn parallel_dir_load(dir: &PathBuf) -> Arc<Mutex<HashMap<String, Vec<String>>>> {
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
