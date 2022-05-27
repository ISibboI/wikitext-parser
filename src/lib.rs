#![warn(missing_docs)]

use error::Result;

mod error;
#[cfg(test)]
mod tests;

/// The root of a wikitext document.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Wikitext;

/// Parse textual wikitext into a semantic representation.
pub fn parse_wikitext(wikitext: &str) -> Result<Wikitext> {
    todo!()
}