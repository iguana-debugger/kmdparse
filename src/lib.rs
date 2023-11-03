use nom::{bytes::complete::tag, combinator::map_res, number::complete::hex_u32, IResult};

/// Parses the KMD tag at the start of every .kmd file
fn kmd_tag(input: &str) -> IResult<&str, &str> {
    tag("KMD\n")(input)
}

fn hex_to_int(input: &str) -> Result<u32, std::num::ParseIntError> {
    u32::from_str_radix(input, 16)
}

fn memory_address(input: &str) -> IResult<&str, u32> {
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
    use nom_test_helpers::{
        assert_done_and_eq, assert_error, assert_error_and_eq, assert_finished,
    };

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
}
