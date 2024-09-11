use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Music {
    pub name: String,
    pub path: PathBuf,
    pub duration: f32,
}
