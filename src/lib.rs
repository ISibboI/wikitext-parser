//! Parse wikitext into a semantic representation.

#![warn(missing_docs)]

use crate::error::Error;
use crate::tokenizer::{MultipeekTokenizer, Token, Tokenizer};
use error::Result;
#[cfg(serde)]
use serde::{Deserialize, Serialize};

mod error;
#[cfg(test)]
mod tests;
mod tokenizer;

const MAX_SECTION_DEPTH: usize = 6;

/// The root of a wikitext document.
#[derive(Debug, Eq, PartialEq, Clone)]
#[cfg_attr(serde, derive(Serialize, Deserialize))]
pub struct Wikitext {
    /// The root of the section tree of the page.
    pub root_section: Section,
}

impl Wikitext {
    /// Print the headlines of the text.
    pub fn print_headlines(&self) {
        self.root_section.print_headlines();
    }
}

/// A section of wikitext.
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(serde, derive(Serialize, Deserialize))]
pub struct Section {
    /// The headline of the section.
    pub headline: Headline,
    /// The text of the section.
    pub text: Vec<TextPiece>,
    /// The subsections of the section.
    pub subsections: Vec<Section>,
}

impl Section {
    /// Print the headlines of the text.
    pub fn print_headlines(&self) {
        println!("{0} {1} {0}", "=".repeat(self.headline.level.into()), self.headline.label);
        for subsection in &self.subsections {
            subsection.print_headlines();
        }
    }
}

/// A headline of a section of wikitext.
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(serde, derive(Serialize, Deserialize))]
pub struct Headline {
    /// The label of the headline.
    pub label: String,
    /// The level of the headline.
    pub level: u8,
}

/// A piece of text of a section of wikitext.
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(serde, derive(Serialize, Deserialize))]
pub enum TextPiece {
    /// A plain string.
    Text(String),
    /// A double brace expression.
    DoubleBraceExpression(Vec<TextPiece>),
}

/// Data structure used to parse wikitext sections and headlines at different levels.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct LevelStack {
    stack: Vec<Vec<Section>>,
}

impl LevelStack {
    /// Create a new headline for a page with the given headline.
    pub fn new(headline: String) -> Self {
        Self {
            stack: vec![vec![Section {
                headline: Headline {
                    label: headline,
                    level: 1,
                },
                text: Vec::new(),
                subsections: vec![],
            }]],
        }
    }

    fn top_mut(&mut self) -> &mut Vec<Section> {
        self.stack.last_mut().unwrap()
    }

    /// Append a new headline found on the page.
    pub fn append_headline(&mut self, headline: Headline) {
        self.adjust_level(headline.level.into());
        debug_assert!(self.stack.len() > 1);
        self.top_mut().push(Section {
            headline,
            text: Vec::new(),
            subsections: vec![],
        });
    }

    /// Append a text piece found on the page.
    pub fn append_text_piece(&mut self, text_piece: TextPiece) {
        self.top_mut().last_mut().unwrap().text.push(text_piece);
    }

    fn adjust_level(&mut self, level: usize) {
        while self.stack.len() > level {
            let mut last = self.stack.pop().unwrap();
            while self.stack.last().unwrap().is_empty() {
                self.stack.pop();
            }
            self.top_mut()
                .last_mut()
                .unwrap()
                .subsections
                .append(&mut last);
        }
        while self.stack.len() < level {
            self.stack.push(Vec::new());
        }
        debug_assert_eq!(self.stack.len(), level);
    }

    /// Collapse the stack down to the root section and return it.
    /// The root section contains the whole section hierarchy added to the stack.
    pub fn to_root_section(mut self) -> Section {
        self.adjust_level(1);
        debug_assert_eq!(self.stack.len(), 1);
        let mut level_1 = self.stack.pop().unwrap();
        debug_assert_eq!(level_1.len(), 1);
        level_1.pop().unwrap()
    }
}

/// Parse textual wikitext into a semantic representation.
pub fn parse_wikitext(wikitext: &str, headline: String) -> Result<Wikitext> {
    let mut level_stack = LevelStack::new(headline);
    let mut tokenizer = MultipeekTokenizer::new(Tokenizer::new(wikitext));

    loop {
        match tokenizer.next() {
            Token::Text(text) => level_stack.append_text_piece(TextPiece::Text(text.to_string())),
            Token::MultiEquals(count) => {
                if let Some(headline) = parse_potential_headline(&mut tokenizer, count) {
                    if headline.level == 1 {
                        return Err(Error::SecondRootSection {
                            label: headline.label,
                        });
                    }
                    level_stack.append_headline(headline);
                } else {
                    level_stack
                        .append_text_piece(TextPiece::Text(Token::MultiEquals(count).to_string()));
                }
            }
            Token::DoubleOpenBrace => {
                level_stack.append_text_piece(parse_double_brace_expression(&mut tokenizer)?)
            }
            Token::DoubleCloseBrace => return Err(Error::UnmatchedDoubleCloseBrace),
            Token::Eof => break,
        }
    }

    Ok(Wikitext {
        root_section: level_stack.to_root_section(),
    })
}

fn parse_potential_headline(tokenizer: &mut MultipeekTokenizer, level: u8) -> Option<Headline> {
    tokenizer.peek(1);
    if let (Some(Token::Text(text)), Some(Token::MultiEquals(second_level))) =
        (tokenizer.repeek(0), tokenizer.repeek(1))
    {
        if level == *second_level && !text.contains(r"\n") {
            let label = text.trim().to_string();
            tokenizer.next();
            tokenizer.next();
            Some(Headline {
                label,
                level,
            })
        } else {
            None
        }
    } else {
        None
    }
}

fn parse_double_brace_expression(tokenizer: &mut MultipeekTokenizer) -> Result<TextPiece> {
    let mut result = Vec::new();

    loop {
        match tokenizer.next() {
            token @ (Token::Text(_) | Token::MultiEquals(_)) => {
                if let Some(TextPiece::Text(text)) = result.last_mut() {
                    text.push_str(&token.to_string());
                } else {
                    result.push(TextPiece::Text(token.to_string()));
                }
            }
            Token::DoubleOpenBrace => result.push(parse_double_brace_expression(tokenizer)?),
            Token::DoubleCloseBrace => break,
            Token::Eof => return Err(Error::UnmatchedDoubleOpenBrace),
        }
    }

    Ok(TextPiece::DoubleBraceExpression(result))
}
