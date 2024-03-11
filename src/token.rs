use crate::{label::Label, line::Line};

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
pub enum Token {
    Tag,
    Line { line: Line },
    Label { label: Label },
}
