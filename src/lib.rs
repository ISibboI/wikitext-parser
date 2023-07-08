//! Parse wikitext into a semantic representation.

#![warn(missing_docs)]

use log::warn;
#[cfg(serde)]
use serde::{Deserialize, Serialize};

mod error;
mod level_stack;
mod parser;
#[cfg(test)]
mod tests;
mod tokenizer;
mod wikitext;

pub use parser::parse_wikitext;
pub use wikitext::{Attribute, Headline, Section, Text, TextFormatting, TextPiece, Wikitext};
