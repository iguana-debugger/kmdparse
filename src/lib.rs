mod label;
mod line;
mod token;

use label::Label;
use line::Line;
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while},
    character::{
        complete::{alpha0, char, multispace0},
        is_hex_digit, is_space,
    },
    combinator::{opt, value},
    multi::many1,
    IResult,
};
use token::Token;

/// Takes the comment section of a KMD line. This parser basically just takes everything up until a
/// newline, trimming the newline in the process. Note that \r\n will probably do weird things here.
fn comment(input: &str) -> IResult<&str, &str> {
    take_while(|c| c != '\r' && c != '\n')(input)
}

fn hex_to_int(input: &str) -> Result<u32, std::num::ParseIntError> {
    let input_no_space = input.replace(" ", "");
    u32::from_str_radix(&input_no_space, 16)
}

fn hex(input: &str) -> IResult<&str, u32> {
    let (remaining, hex): (&str, &str) =
        take_while(|c: char| is_hex_digit(c as u8) || c == ' ')(input)?;

    let res = hex_to_int(hex).map_err(|_| {
        nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::HexDigit,
        ))
    })?;

    Ok((remaining, res))
}

/// Parses the KMD tag at the start of every .kmd file
fn kmd_tag(input: &str) -> IResult<&str, &str> {
    alt((tag("KMD\r\n"), tag("KMD\n")))(input)
}

fn label(input: &str) -> IResult<&str, Label> {
    // Take the leading colon and space
    let (remaining, _) = tag(": ")(input)?;

    // Take the name (any alphanumeric character)
    let (remaining, name) = alpha0(remaining)?; // Can you have functions with numbers?

    // Take the trailing spaces
    let (remaining, _) = multispace0(remaining)?;

    let (remaining, memory_address) = hex(remaining)?;

    let (remaining, _) = multispace0(remaining)?;

    // aasm is really sneaky about adding two dashes to local
    let (remaining, is_exported) = alt((
        value(true, tag("Global - ")),
        value(false, tag("Local -- ")),
    ))(remaining)?;

    let (remaining, is_thumb) =
        alt((value(true, tag("Thumb")), value(false, tag("ARM"))))(remaining)?;

    // Take the newline off the end
    let (remaining, _) = alt((tag("\r\n"), tag("\n")))(remaining)?;

    Ok((
        remaining,
        Label::new(name.to_string(), memory_address, is_exported, is_thumb),
    ))
}

fn label_title(input: &str) -> IResult<&str, &str> {
    alt((
        tag("Symbol Table: Labels\n"),
        tag("Symbol Table: Labels\r\n"),
    ))(input)
}

fn line(input: &str) -> IResult<&str, Token> {
    if let Ok((remaining, _)) = kmd_tag(input) {
        return Ok((remaining, Token::Tag));
    }

    let (remaining, memory_address) = hex(input)?;
    let (remaining, _) = char(':')(remaining)?;
    let (remaining, _) = take_while(|c| is_space(c as u8))(remaining)?;
    let (remaining, word) = opt(hex)(remaining)?;
    let (remaining, _) = take_while(|c| is_space(c as u8))(remaining)?;
    let (remaining, _) = char(';')(remaining)?;
    let (remaining, comment) = comment(remaining)?;
    let (remaining, _) = alt((tag("\r\n"), tag("\n")))(remaining)?;

    Ok((
        remaining,
        Token::Line(Line::new(memory_address, word, comment.to_string())),
    ))
}

pub fn parse_kmd(input: &str) -> IResult<&str, Vec<Token>> {
    let (_, _) = kmd_tag(input)?;

    let (remaining, lines) = many1(line)(input)?;

    Ok((remaining, lines))
}

#[cfg(test)]
mod tests {
    use nom_test_helpers::{
        assert_done_and_eq, assert_error, assert_finished, assert_finished_and_eq,
    };
    use rayon::prelude::{IntoParallelIterator, ParallelIterator};

    use super::*;

    static EXAMPLE: &'static str = include_str!("hello.kmd");
    static EXAMPLE_LINES: usize = 22;

    #[test]
    fn test_kmd_tag_valid() {
        assert_finished!(kmd_tag("KMD\n"));
    }

    #[test]
    fn test_kmd_tag_invalid() {
        // RIP IGU format
        assert_error!(kmd_tag("IGU\n"));
    }

    #[test]
    fn test_kmd_tag_valid_extra() {
        let (remaining, tag) = kmd_tag("KMD\nextra").unwrap();
        assert_eq!(tag, "KMD\n");
        assert_eq!(remaining, "extra");
    }

    #[test]
    fn test_hex_valid() {
        (0..u32::MAX)
            .into_par_iter()
            .for_each(|i| assert_done_and_eq!(hex(&format!("{:X}", i)), i));
    }

    #[test]
    fn test_hex_invalid() {
        assert_error!(hex("notamemoryaddress"));
        assert_error!(hex(""));
    }

    #[test]
    fn test_hex_overflow() {
        assert_error!(hex("FFFFFFFFFFFFFFFF"));
    }

    #[test]
    fn test_hex_spaces() {
        assert_done_and_eq!(hex("DE AD BE EF"), 0xDEADBEEF);
        assert_done_and_eq!(hex("0A 00"), 0x0A00);
    }

    #[test]
    fn test_comment() {
        let comment_text = "Hello\n";
        assert_done_and_eq!(comment(comment_text), "Hello");
    }

    #[test]
    fn test_line_tag() {
        assert_done_and_eq!(line("KMD\n"), Token::Tag);
    }

    #[test]
    fn test_line_line() {
        let expected = Line::new(
            0x00000008,
            Some(0x42757A7A),
            " buzz    DEFB \"Buzz\",0".to_string(),
        );

        assert_done_and_eq!(
            line("00000008: 42 75 7A 7A ; buzz    DEFB \"Buzz\",0\n"),
            Token::Line(expected)
        );
    }

    #[test]
    fn test_parse_kmd_valid() {
        let (_, lines) = parse_kmd(EXAMPLE).unwrap();

        assert_eq!(lines.len(), EXAMPLE_LINES)
    }

    #[test]
    fn test_parse_kmd_valid_crlf() {
        let example = include_str!("hello.kmd").replace("\n", "\r\n");
        let (_, lines) = parse_kmd(&example).unwrap();

        assert_eq!(lines.len(), EXAMPLE_LINES)
    }

    #[test]
    fn test_label_valid_local_arm() {
        let input = ": hello                             00000004  Local -- ARM\n";
        let expected = Label::new("hello".to_string(), 0x00000004, false, false);

        assert_finished_and_eq!(label(input), expected);
    }

    #[test]
    fn test_label_valid_exported_arm() {
        let input = ": hello                             00000004  Global - ARM\n";
        let expected = Label::new("hello".to_string(), 0x00000004, true, false);

        assert_finished_and_eq!(label(input), expected);
    }

    #[test]
    fn test_label_valid_local_thumb() {
        let input = ": hello                             00000004  Local -- Thumb\n";
        let expected = Label::new("hello".to_string(), 0x00000004, false, true);

        assert_finished_and_eq!(label(input), expected);
    }

    #[test]
    fn test_label_valid_exported_thumb() {
        let input = ": hello                             00000004  Global - Thumb\n";
        let expected = Label::new("hello".to_string(), 0x00000004, true, true);

        assert_finished_and_eq!(label(input), expected);
    }
}
