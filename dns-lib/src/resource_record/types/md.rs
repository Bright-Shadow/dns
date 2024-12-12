use dns_macros::{ToWire, FromWire, FromTokenizedRData, RData, ToPresentation};

use crate::types::c_domain_name::CDomainName;

/// (Original) https://datatracker.ietf.org/doc/html/rfc1035#section-3.3.4
#[derive(Clone, PartialEq, Eq, Hash, Debug, ToWire, FromWire, ToPresentation, FromTokenizedRData, RData)]
pub struct MD {
    ma_domain_name: CDomainName,
}

impl MD {
    #[inline]
    pub fn new(ma_domain_name: CDomainName) -> Self {
        Self { ma_domain_name }
    }

    #[inline]
    pub fn mail_agent_domain_name(&self) -> &CDomainName {
        &self.ma_domain_name
    }
}

#[cfg(test)]
mod circular_serde_sanity_test {
    use crate::{serde::wire::circular_test::gen_test_circular_serde_sanity_test, types::c_domain_name::CDomainName};
    use super::MD;

    gen_test_circular_serde_sanity_test!(
        record_circular_serde_sanity_test,
        MD { ma_domain_name: CDomainName::from_utf8("www.example.com.").unwrap() }
    );
}

#[cfg(test)]
mod tokenizer_tests {
    use crate::{serde::presentation::test_from_tokenized_rdata::{gen_ok_record_test, gen_fail_record_test}, types::c_domain_name::CDomainName};
    use super::MD;

    const GOOD_DOMAIN: &str = "www.example.com.";
    const BAD_DOMAIN: &str = "..www.example.com.";

    gen_ok_record_test!(test_ok, MD, MD { ma_domain_name: CDomainName::from_utf8(GOOD_DOMAIN).unwrap() }, [GOOD_DOMAIN]);

    gen_fail_record_test!(test_fail_bad_domain, MD, [BAD_DOMAIN]);
    gen_fail_record_test!(test_fail_two_tokens, MD, [GOOD_DOMAIN, GOOD_DOMAIN]);
    gen_fail_record_test!(test_fail_no_tokens, MD, []);
}
