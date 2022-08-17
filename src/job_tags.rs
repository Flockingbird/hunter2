pub fn tags(file_name: &str) -> Vec<String> {
    read_all(file_name)
}

fn read_all(file_name: &str) -> Vec<String> {
    std::fs::read_to_string(file_name)
        .unwrap_or_else(|_| panic!("file not found: {}", file_name))
        .lines()
        .map(|x| x.parse().expect("cannot read file contents"))
        .collect()
}
