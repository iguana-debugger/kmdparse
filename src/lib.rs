use nom::{
    bytes::complete::{tag, take_while},
    character::complete::hex_digit1,
    combinator::map_res,
    number::complete::hex_u32,
    AsChar, IResult, Parser,
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

    // // Removes the whitespace by splitting by whitespace and sticking the splits back together
    // let hex_no_space = hex.split(' ').collect::<Vec<_>>().concat();

    // // The hex_digit1 is a bit redundant here, but oh well
    // let (_, res) = map_res(&hex_no_space, hex_to_int)(&hex_no_space)?;

    let res = hex
        .split(' ')
        .map(|hex_str| {
            hex_to_int(hex_str).map_err(|_| {
                nom::Err::Error(nom::error::Error::new(
                    input,
                    nom::error::ErrorKind::HexDigit,
                ))
            })
        })
        .product::<Result<u32, _>>()?; // Need to do binary shifty stuff

    Ok((remaining, res))
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
            .for_each(|i| assert_done_and_eq!(hex(&format!("{:X}", i)), i))
    }

    #[test]
    fn test_hex_invalid() {
        assert_error!(hex("notamemoryaddress"));
    }

    #[test]
    fn test_hex_overflow() {
        assert_error!(hex("FFFFFFFFFFFFFFFF"))
    }

    #[test]
    fn test_hex_spaces() {
        assert_done_and_eq!(hex("DE AD BE EF"), 0xDEADBEEF)
    }
}
