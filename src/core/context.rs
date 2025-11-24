use std::path::PathBuf;
#[derive(Debug, Clone)]
pub struct AppContext {
    pub main_file_path: PathBuf,
}
impl AppContext {
    pub fn new(main_file_path: PathBuf) -> Self {
        Self { main_file_path }
    }
}
