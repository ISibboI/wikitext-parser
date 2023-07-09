//! Parse wikitext into a semantic representation.

#![warn(missing_docs)]

mod error;
mod level_stack;
mod parser;
#[cfg(test)]
mod tests;
mod tokenizer;
mod wikitext;

pub use error::{ParserError, ParserErrorKind};
pub use parser::parse_wikitext;
pub use tokenizer::TextPosition;
pub use wikitext::{Attribute, Headline, Section, Text, TextFormatting, TextPiece, Wikitext};
