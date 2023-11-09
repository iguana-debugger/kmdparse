#[derive(Debug, PartialEq)]
pub struct Line {
    memory_address: u32,
    word: Option<u32>,
    comment: String,
}

impl Line {
    pub fn new(memory_address: u32, word: Option<u32>, comment: String) -> Self {
        Self {
            memory_address,
            word,
            comment,
        }
    }
}
