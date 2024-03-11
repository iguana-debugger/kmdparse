use crate::word::Word;

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct Line {
    pub memory_address: Option<u32>,
    pub word: Option<Word>,
    pub comment: String,
}

impl Line {
    pub fn new(memory_address: Option<u32>, word: Option<Word>, comment: String) -> Self {
        Self {
            memory_address,
            word,
            comment,
        }
    }
}
