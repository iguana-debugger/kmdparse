use crate::{label::Label, line::Line};

#[derive(Debug, PartialEq)]
pub enum Token {
    Tag,
    Line(Line),
    Label(Label),
}
