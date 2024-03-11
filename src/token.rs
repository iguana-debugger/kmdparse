use crate::{label::Label, line::Line};

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Tag,
    Line(Line),
    Label(Label),
}
