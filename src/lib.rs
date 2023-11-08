use nom::{
    branch::alt,
    bytes::complete::{tag, take_while},
    character::{complete::hex_digit1, is_hex_digit},
    combinator::{map_res, recognize},
    number::complete::hex_u32,
    AsChar, IResult, Parser,
};

type Opcode = u32;

fn hex_to_int(input: &str) -> Result<u32, std::num::ParseIntError> {
    let input_no_space = input.replace(" ", "");
    u32::from_str_radix(&input_no_space, 16)
}

fn hex(input: &str) -> IResult<&str, u32> {
    let (remaining, hex): (&str, &str) =
        take_while(|c: char| is_hex_digit(c as u8) || c == ' ')(input)?;

    // // Removes the whitespace by splitting by whitespace and sticking the splits back together
    // let hex_no_space = hex.replace(" ", "");

    // // The hex_digit1 is a bit redundant here, but oh well
    // let (_, res) = map_res(hex_digit1, hex_to_int)(hex)?;

    let res = hex_to_int(hex).map_err(|_| {
        nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::HexDigit,
        ))
    })?;

    // let res = hex.split(' ').map(|hex_str| {
    //     hex_to_int(hex_str).map_err(|_| {
    //         nom::Err::Error(nom::error::Error::new(
    //             input,
    //             nom::error::ErrorKind::HexDigit,
    //         ))
    //     })
    // });

    Ok((remaining, res))
}

/// Parses the KMD tag at the start of every .kmd file
fn kmd_tag(input: &str) -> IResult<&str, &str> {
    tag("KMD\n")(input)
}

pub fn parse_kmd(input: &str) {
    // let (remaining, _) = kmd_tag(input)?;
}

#[cfg(test)]
mod tests {
    use nom_test_helpers::{assert_done_and_eq, assert_error, assert_finished};
    use rayon::prelude::{IntoParallelIterator, ParallelIterator};

    use super::*;

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
}
