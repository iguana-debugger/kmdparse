use crate::line::Line;

#[derive(Debug, PartialEq)]
pub enum Token {
    Tag,
    Line(Line),
}
