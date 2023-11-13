#[derive(Debug, PartialEq)]
pub struct Label {
    /// The name of the label
    pub name: String,

    /// The associated memory address of the label
    pub memory_address: u32,

    /// Whether or not the label is global (true for global, false for local)
    pub is_exported: bool,

    /// Whether or not the label points to a Thumb instruction
    pub is_thumb: bool,
}

impl Label {
    pub fn new(name: String, memory_address: u32, is_exported: bool, is_thumb: bool) -> Self {
        Self {
            name,
            memory_address,
            is_exported,
            is_thumb,
        }
    }
}
