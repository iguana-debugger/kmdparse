use nom::{
    bytes::complete::{tag, take_while},
    combinator::map_res,
    number::complete::hex_u32,
    AsChar, IResult,
};

type Opcode = u32;

/// Parses the KMD tag at the start of every .kmd file
fn kmd_tag(input: &str) -> IResult<&str, &str> {
    tag("KMD\n")(input)
}

fn hex_to_int(input: &str) -> Result<u32, std::num::ParseIntError> {
    u32::from_str_radix(input, 16)
}

fn hex(input: &str) -> IResult<&str, u32> {
    let (remaining, hex): (&str, &str) = take_while(|c: char| c.is_alphanum() || c == ' ')(input)?;

    map_res(
        nom::bytes::complete::take_while(|c: char| c.is_ascii_hexdigit()),
        hex_to_int,
    )(input)
}

pub fn parse_kmd(input: &str) {
    // let (remaining, _) = kmd_tag(input)?;
}

#[cfg(test)]
mod tests {
    use nom_test_helpers::{assert_done_and_eq, assert_error, assert_finished};

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
        assert_done_and_eq!(hex("0000000C"), 12)
    }

    #[test]
    fn test_hex_invalid() {
        assert_error!(hex("notamemoryaddress"));
    }

    #[test]
    fn test_hex_overflow() {
        assert_error!(hex("FFFFFFFFFFFFFFFF"))
    }
}
