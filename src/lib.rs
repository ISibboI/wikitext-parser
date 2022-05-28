//! Parse wikitext into a semantic representation.

#![warn(missing_docs)]

use error::Result;
#[cfg(serde)]
use serde::{Deserialize, Serialize};

mod error;
#[cfg(test)]
mod tests;

/// The root of a wikitext document.
#[derive(Debug, Eq, PartialEq, Clone)]
#[cfg_attr(serde, derive(Serialize, Deserialize))]
pub struct Wikitext {
    root_section: Section,
}

/// A section of wikitext.
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(serde, derive(Serialize, Deserialize))]
pub struct Section {
    text: String,
    level: u8,
    subsections: Vec<Section>,
}

/// Parse textual wikitext into a semantic representation.
pub fn parse_wikitext(wikitext: &str) -> Result<Wikitext> {
    Ok(Wikitext {
        root_section: parse_wikitext_recursively(wikitext, 0)?,
    })
}

fn parse_wikitext_recursively(text: &str, level: u8) -> Result<Section> {
    if let Some((text, label, new_level)) = find_next_headline(text)? {}
    todo!()
}

fn find_next_headline(text: &str) -> Result<Option<(&str, String, u8)>> {
    if let Some(location) = text.find('=') {
        let text = &text[location..];

        todo!()
    } else {
        Ok(None)
    }
}
