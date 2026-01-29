#[derive(Debug, Clone)]
pub struct ChangedFile {
    pub path: String,
    pub changed_lines: Vec<u32>,
}
