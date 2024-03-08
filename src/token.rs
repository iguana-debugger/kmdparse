use crate::{label::Label, line::Line};

#[derive(Clone, Debug, PartialEq, uniffi::Enum)]
pub enum Token {
    Tag,
    Line { line: Line },
    Label { label: Label },
}
