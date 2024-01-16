use std::fmt::Display;

use super::{errors::TokenizerError, entry_text_tokens::{EntryTokenIter, EntryTextToken}, regex::{REGEX_RCLASS, REGEX_RTYPE, REGEX_TTL}};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Entry<'a> {
    Origin{origin: &'a str},
    Include{file_name: &'a str, domain_name: Option<&'a str>},
    ResourceRecord{domain_name: Option<&'a str>, ttl: Option<&'a str>, rclass: Option<&'a str>, rtype: &'a str, rdata: Vec<&'a str>},
}

impl<'a> Display for Entry<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Entry::Origin{origin} => {
                writeln!(f, "ORIGIN")?;
                writeln!(f, "\tDomain Name: {origin}")
            },
            Entry::Include{file_name, domain_name} => {
                writeln!(f, "INCLUDE")?;
                writeln!(f, "\tFile Name: {file_name}")?;
                if let Some(domain_name) = domain_name {
                    writeln!(f, "\tDomain Name: {domain_name}")?;
                }
                Ok(())
            },
            Entry::ResourceRecord{domain_name, ttl, rclass, rtype, rdata} => {
                writeln!(f, "Resource Record")?;
                if let Some(domain_name) = domain_name {
                    writeln!(f, "\tDomain Name: {}", domain_name)?;
                }
                if let Some(ttl) = ttl {
                    writeln!(f, "\tTTL: {}", ttl)?;
                }
                if let Some(rclass) = rclass {
                    writeln!(f, "\tClass: {}", rclass)?;
                }
                writeln!(f, "\tType: {}", rtype)?;
                for rdata in rdata {
                    writeln!(f, "\tRData: {}", rdata)?;
                }
                Ok(())
            },
        }
    }
}

pub struct EntryIter<'a> {
    token_iter: EntryTokenIter<'a>
}

impl<'a> EntryIter<'a> {
    #[inline]
    pub fn new(feed: &'a str) -> Self {
        EntryIter { token_iter: EntryTokenIter::new(feed) }
    }
}

impl<'a> Iterator for EntryIter<'a> {
    type Item = Result<Entry<'a>, TokenizerError<'a>>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let entry_tokens = match self.token_iter.next() {
                Some(Ok(entry_tokens)) => entry_tokens,
                Some(Err(error)) => return Some(Err(error)),
                None => return None,
            };

            match entry_tokens.text_tokens.as_slice() {
                // <blank>[<comment>]
                &[] => continue,    //< Skip entries that are empty
    
                // $ORIGIN <domain-name> [<comment>]
                &[EntryTextToken::TextLiteral("$ORIGIN"), EntryTextToken::TextLiteral(domain_name)] => return Some(Ok(
                    Entry::Origin{ origin: domain_name }
                )),
    
                // $INCLUDE <file-name> [<domain-name>] [<comment>]
                &[EntryTextToken::TextLiteral("$INCLUDE"), EntryTextToken::TextLiteral(file_name)] => return Some(Ok(
                    Entry::Include{ file_name, domain_name: None }
                )),
                &[EntryTextToken::TextLiteral("$INCLUDE"), EntryTextToken::TextLiteral(file_name), EntryTextToken::TextLiteral(domain_name)] => return Some(Ok(
                    Entry::Include{ file_name, domain_name: Some(domain_name) }
                )),
    
                // <domain-name> [<TTL>] [<class>] <type> <RDATA> [<comment>]
                &[EntryTextToken::TextLiteral(domain_name), ..] => return Some(Self::parse_rr(Some(domain_name), &entry_tokens.text_tokens[1..])),
                // <blank> [<TTL>] [<class>] <type> <RDATA> [<comment>]
                &[EntryTextToken::Separator(_), ..] => return Some(Self::parse_rr(None, &entry_tokens.text_tokens[1..])),
            }
        }
    }
}

impl<'a> EntryIter<'a> {
    #[inline]
    fn new_rr<'b>(domain_name: Option<&'a str>, ttl: Option<&'a str>, rclass: Option<&'a str>, rtype: &'a str, rdata: impl Iterator<Item = &'b EntryTextToken<'a>>) -> Entry<'a> where 'a: 'b {
        Entry::ResourceRecord{
            domain_name,
            ttl,
            rclass,
            rtype,
            rdata: rdata.map(|token| token.into()).collect(),
        }
    }

    #[inline]
    fn parse_rr(domain_name: Option<&'a str>, other_tokens: &[EntryTextToken<'a>]) -> Result<Entry<'a>, TokenizerError<'a>> {
        match other_tokens {
            &[EntryTextToken::TextLiteral(token_1), EntryTextToken::TextLiteral(token_2), ..] => {
                // If the first token is an rtype, then the rest is the rdata and we should not read it
                if (!REGEX_RCLASS.is_match(token_1)) && REGEX_RTYPE.is_match(token_1) {
                    return Self::parse_rr_rtype_first(domain_name, token_1, &other_tokens[1..]);
                }

                if (!REGEX_RCLASS.is_match(token_2)) && REGEX_RTYPE.is_match(token_2) {
                    return Self::parse_rr_rtype_second(domain_name, token_1, token_2, &other_tokens[2..]);
                }

                // The match case only covers a minimum of 2 tokens. This case can only happen if
                // there are at least 3.
                if other_tokens.len() >= 3 {
                    let token_3 = other_tokens[2].into();
                    if (!REGEX_RCLASS.is_match(token_3)) && REGEX_RTYPE.is_match(token_3) {
                        return Self::parse_rr_rtype_third(domain_name, token_1, token_2, token_3, &other_tokens[3..]);
                    } else {
                        return Err(TokenizerError::UnknownToken(token_3));
                    }
                }

                return Err(TokenizerError::TwoUnknownTokens(token_1, token_2));
            },
            _ => return Err(TokenizerError::UnknownTokens),
        }
    }

    #[inline]
    fn parse_rr_rtype_first(domain_name: Option<&'a str>, rtype: &'a str, other_tokens: &[EntryTextToken<'a>]) -> Result<Entry<'a>, TokenizerError<'a>> {
        Ok(Self::new_rr(
            domain_name,
            None,
            None,
            rtype,
            other_tokens.iter()
        ))
    }

    #[inline]
    fn parse_rr_rtype_second(domain_name: Option<&'a str>, token_1: &'a str, rtype: &'a str, other_tokens: &[EntryTextToken<'a>]) -> Result<Entry<'a>, TokenizerError<'a>> {
        if REGEX_RCLASS.is_match(token_1) {
            Ok(Self::new_rr(
                domain_name,
                None,
                Some(token_1),
                rtype,
                other_tokens.iter()
            ))
        } else if REGEX_TTL.is_match(token_1) {
            Ok(Self::new_rr(
                domain_name,
                Some(token_1),
                None,
                rtype,
                other_tokens.iter()
            ))
        } else {
            Err(TokenizerError::UnknownToken(token_1))
        }
    }

    #[inline]
    fn parse_rr_rtype_third(domain_name: Option<&'a str>, token_1: &'a str, token_2: &'a str, rtype: &'a str, other_tokens: &[EntryTextToken<'a>]) -> Result<Entry<'a>, TokenizerError<'a>> {
        if REGEX_RCLASS.is_match(token_1) && REGEX_TTL.is_match(token_2) {
            Ok(Self::new_rr(
                domain_name,
                Some(token_2),
                Some(token_1),
                rtype,
                other_tokens.iter()
            ))
        } else if REGEX_TTL.is_match(token_1) && REGEX_RCLASS.is_match(token_2) {
            Ok(Self::new_rr(
                domain_name,
                Some(token_1),
                Some(token_2),
                rtype,
                other_tokens.iter()
            ))
        } else {
            Err(TokenizerError::TwoUnknownTokens(token_1, token_2))
        }
    }
}
