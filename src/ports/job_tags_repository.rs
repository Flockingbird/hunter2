pub trait JobTagsRepository {
    fn tags(&self) -> Vec<String>;
}

pub struct JobTagsFileRepository {
    tags: Vec<String>,
}

impl JobTagsFileRepository {
    pub fn new(file_name: String) -> Self {
        let tags = read_all(&file_name);
        Self { tags }
    }
}

impl JobTagsRepository for JobTagsFileRepository {
    fn tags(&self) -> Vec<String> {
        self.tags.clone()
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
