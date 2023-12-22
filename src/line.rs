#[derive(Debug, PartialEq)]
pub struct Line {
    pub memory_address: u32,
    pub word: Option<Vec<u8>>,
    pub comment: String,
}

impl Line {
    pub fn new(memory_address: u32, word: Option<Vec<u8>>, comment: String) -> Self {
        Self {
            memory_address,
            word,
            comment,
        }
    }
}
