#[derive(Debug, PartialEq)]
pub struct Line {
    pub memory_address: u32,
    pub word: Option<u32>,
    pub comment: String,
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
