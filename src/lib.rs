pub mod label;
pub mod line;
pub mod token;
pub mod word;

#[cfg(feature = "uniffi")]
mod uniffi_array;

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
    multi::{many0, many1},
    AsChar, IResult,
};
use token::Token;
use word::Word;

#[cfg(feature = "uniffi")]
uniffi::setup_scaffolding!();

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

fn label(input: &str) -> IResult<&str, Token> {
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
        Token::Label {
            label: Label::new(name.to_string(), memory_address, is_exported, is_thumb),
        },
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

    let (remaining, memory_address) = opt(hex)(input)?;
    let (remaining, _) = opt(char(':'))(remaining)?;
    let (remaining, _) = take_while(|c| is_space(c as u8))(remaining)?;
    let (remaining, word) = opt(word)(remaining)?;
    let (remaining, _) = take_while(|c| is_space(c as u8))(remaining)?;
    let (remaining, _) = char(';')(remaining)?;
    let (remaining, comment) = comment(remaining)?;
    let (remaining, _) = alt((tag("\r\n"), tag("\n")))(remaining)?;

    Ok((
        remaining,
        Token::Line {
            line: Line::new(memory_address, word, comment.to_string()),
        },
    ))
}

fn word(input: &str) -> IResult<&str, Word> {
    let (remaining, hex_digits_untrimmed) =
        take_while(|c: char| c.is_hex_digit() || c == ' ')(input)?;
    let hex_digits = hex_digits_untrimmed.trim_end();

    let contains_whitespace = !hex_digits
        .chars()
        .collect::<Vec<_>>()
        .windows(8)
        .any(|window| window.iter().all(|c| !c.is_whitespace()));

    let hex_digits_no_space = hex_digits
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<Vec<_>>();

    // let parsed = hex_digits_no_space
    //     .chunks(2)
    //     .try_map(|byte_chars| byte_chars.iter().collect::<String>())
    //     .try_map(|byte_str| {
    //         u8::from_str_radix(&byte_str, 16).map_err(|e| {
    //             nom::Err::Error(nom::error::Error::new(
    //                 input,
    //                 nom::error::ErrorKind::HexDigit,
    //             ))
    //         })
    //     })?
    //     .collect::<Vec<_>>();

    let mut parsed = vec![];

    for chunk in hex_digits_no_space.chunks(2) {
        let chunk_str = chunk.iter().collect::<String>();

        let hex = u8::from_str_radix(&chunk_str, 16).map_err(|_| {
            nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::HexDigit,
            ))
        })?;

        parsed.push(hex);
    }

    // If the word contained whitespace, we know that it isn't an instruction.
    let word = if contains_whitespace {
        Word::Data { data: parsed }
    } else {
        // The KMD file format stores instructions backwards for reasons that I don't quite
        // understand, so we flip the bytes around (and convert the vec into an array) here.
        let instruction = parsed
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
            .try_into()
            .map_err(|_| {
                nom::Err::Error(nom::error::Error::new(
                    input,
                    nom::error::ErrorKind::TooLarge,
                ))
            })?;

        Word::Instruction { instruction }
    };

    Ok((remaining, word))
}

pub fn parse_kmd(input: &str) -> IResult<&str, Vec<Token>> {
    let (remaining, _) = kmd_tag(input)?;

    let (remaining, mut lines) = many1(line)(remaining)?;

    let (remaining, _) = alt((tag("\r\n"), tag("\n")))(remaining)?;
    let (remaining, _) = label_title(remaining)?;

    let (remaining, mut labels) = many0(label)(remaining)?;

    lines.append(&mut labels);

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
    static EXAMPLE_LINES: usize = 24;

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
            Some(0x00000008),
            Some(Word::Data {
                data: vec![0x42, 0x75, 0x7A, 0x7A],
            }),
            " buzz    DEFB \"Buzz\",0".to_string(),
        );

        assert_done_and_eq!(
            line("00000008: 42 75 7A 7A ; buzz    DEFB \"Buzz\",0\n"),
            Token::Line { line: expected }
        );
    }

    #[test]
    fn test_word_valid() {
        assert_done_and_eq!(
            word("DEADBEEF"),
            Word::Instruction {
                instruction: [0xEF, 0xBE, 0xAD, 0xDE]
            }
        );

        assert_done_and_eq!(
            word("DE AD BE EF"),
            Word::Data {
                data: vec![0xDE, 0xAD, 0xBE, 0xEF]
            }
        );
    }

    #[test]
    fn test_word_valid_short() {
        // I think "DEAD" would be valid for data?
        assert_done_and_eq!(
            word("DEAD"),
            Word::Data {
                data: vec![0xDE, 0xAD]
            }
        );
        assert_done_and_eq!(
            word("DE AD"),
            Word::Data {
                data: vec![0xDE, 0xAD]
            }
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
        let expected = Token::Label {
            label: Label::new("hello".to_string(), 0x00000004, false, false),
        };

        assert_finished_and_eq!(label(input), expected);
    }

    #[test]
    fn test_label_valid_exported_arm() {
        let input = ": hello                             00000004  Global - ARM\n";
        let expected = Token::Label {
            label: Label::new("hello".to_string(), 0x00000004, true, false),
        };

        assert_finished_and_eq!(label(input), expected);
    }

    #[test]
    fn test_label_valid_local_thumb() {
        let input = ": hello                             00000004  Local -- Thumb\n";
        let expected = Token::Label {
            label: Label::new("hello".to_string(), 0x00000004, false, true),
        };

        assert_finished_and_eq!(label(input), expected);
    }

    #[test]
    fn test_label_valid_exported_thumb() {
        let input = ": hello                             00000004  Global - Thumb\n";
        let expected = Token::Label {
            label: Label::new("hello".to_string(), 0x00000004, true, true),
        };

        assert_finished_and_eq!(label(input), expected);
    }
}
