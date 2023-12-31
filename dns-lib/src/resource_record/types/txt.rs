use dns_macros::{ToWire, FromWire, RTypeCode};

use crate::{types::character_string::CharacterString, serde::presentation::{from_tokenized_record::FromTokenizedRecord, from_presentation::FromPresentation}};

/// (Original) https://datatracker.ietf.org/doc/html/rfc1035#section-3.3.14
#[derive(Clone, PartialEq, Eq, Hash, Debug, ToWire, FromWire, RTypeCode)]
pub struct TXT {
    strings: Vec<CharacterString>,
}

impl TXT {
    #[inline]
    pub fn new(strings: Vec<CharacterString>) -> Self {
        Self { strings }
    }

    #[inline]
    pub fn strings(&self) -> &[CharacterString] {
        &self.strings
    }
}

impl FromTokenizedRecord for TXT {
    #[inline]
    fn from_tokenized_record<'a>(record: &'a crate::serde::presentation::tokenizer::tokenizer::ResourceRecord) -> Result<Self, crate::serde::presentation::errors::TokenizedRecordError<'a>> where Self: Sized {
        match record.rdata.as_slice() {
            &[_, ..] => {
                let mut strings = Vec::with_capacity(record.rdata.len());
                for string_token in &record.rdata {
                    strings.push(CharacterString::from_token_format(&string_token)?);
                }
                Ok(Self { strings })
            },
            _ => Err(crate::serde::presentation::errors::TokenizedRecordError::TooFewRDataTokensError(1, record.rdata.len())),
        }
    }
}

#[cfg(test)]
mod circular_serde_sanity_test {
    use crate::{serde::wire::circular_test::gen_test_circular_serde_sanity_test, types::character_string::CharacterString};
    use super::TXT;

    gen_test_circular_serde_sanity_test!(
        record_single_string_circular_serde_sanity_test,
        TXT {
            strings: vec![
                CharacterString::from_utf8("This string is all alone.").unwrap(),
            ]
        }
    );
    gen_test_circular_serde_sanity_test!(
        record_two_string_circular_serde_sanity_test,
        TXT {
            strings: vec![
                CharacterString::from_utf8("This is a pretty cool string.").unwrap(),
                CharacterString::from_utf8("This string isn't as cool.").unwrap(),
            ]
        }
    );
}


#[cfg(test)]
mod tokenizer_tests {
    use crate::{serde::presentation::test_from_tokenized_record::{gen_ok_record_test, gen_fail_record_test}, types::character_string::CharacterString};
    use super::TXT;

    const GOOD_STRING: &str = "This is a string with some characters";

    gen_ok_record_test!(
        test_ok_one_string,
        TXT,
        TXT { strings: vec![
            CharacterString::from_utf8(GOOD_STRING).unwrap(),
        ] },
        [GOOD_STRING]
    );
    gen_ok_record_test!(
        test_ok_two_string,
        TXT,
        TXT { strings: vec![
            CharacterString::from_utf8(GOOD_STRING).unwrap(),
            CharacterString::from_utf8(GOOD_STRING).unwrap(),
        ] },
        [GOOD_STRING, GOOD_STRING]
    );
    gen_ok_record_test!(
        test_ok_three_string,
        TXT,
        TXT { strings: vec![
            CharacterString::from_utf8(GOOD_STRING).unwrap(),
            CharacterString::from_utf8(GOOD_STRING).unwrap(),
            CharacterString::from_utf8(GOOD_STRING).unwrap(),
        ] },
        [GOOD_STRING, GOOD_STRING, GOOD_STRING]
    );
    gen_ok_record_test!(
        test_ok_four_string,
        TXT,
        TXT { strings: vec![
            CharacterString::from_utf8(GOOD_STRING).unwrap(),
            CharacterString::from_utf8(GOOD_STRING).unwrap(),
            CharacterString::from_utf8(GOOD_STRING).unwrap(),
            CharacterString::from_utf8(GOOD_STRING).unwrap(),
        ] },
        [GOOD_STRING, GOOD_STRING, GOOD_STRING, GOOD_STRING]
    );
    gen_ok_record_test!(
        test_ok_five_string,
        TXT,
        TXT { strings: vec![
            CharacterString::from_utf8(GOOD_STRING).unwrap(),
            CharacterString::from_utf8(GOOD_STRING).unwrap(),
            CharacterString::from_utf8(GOOD_STRING).unwrap(),
            CharacterString::from_utf8(GOOD_STRING).unwrap(),
            CharacterString::from_utf8(GOOD_STRING).unwrap(),
        ] },
        [GOOD_STRING, GOOD_STRING, GOOD_STRING, GOOD_STRING, GOOD_STRING]
    );
    gen_fail_record_test!(test_fail_no_tokens, TXT, []);
}
