pub trait JobTagsRepository {
    fn tags(&self) -> Vec<String>;
}

pub struct JobTagsFileRepository {
    file_name: String,
}

impl JobTagsFileRepository {
    pub fn new(file_name: String) -> Self {
        Self { file_name }
    }
}

impl JobTagsRepository for JobTagsFileRepository {
    fn tags(&self) -> Vec<String> {
        read_all(&self.file_name)
    }
}

pub struct JobTagsMemoryRepository {
    pub tags: Vec<String>,
}

impl JobTagsRepository for JobTagsMemoryRepository {
    fn tags(&self) -> Vec<String> {
        self.tags.clone()
    }
}
fn read_all(file_name: &str) -> Vec<String> {
    std::fs::read_to_string(file_name)
        .unwrap_or_else(|_| panic!("file not found: {}", file_name))
        .lines()
        .map(|x| x.parse().expect("cannot read file contents"))
        .collect()
}
