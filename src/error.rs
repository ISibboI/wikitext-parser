use std::error::Error;
use std::fmt::Display;

use crate::tokenizer::TextPosition;
use crate::wikitext::TextFormatting;

pub type Result<T> = std::result::Result<T, ParserError>;

/// Error type of this crate.
#[derive(Debug, Eq, PartialEq)]
pub struct ParserError {
    /// The kind of error.
    pub kind: ParserErrorKind,
    /// The position of the error in text.
    pub position: TextPosition,
    /// Further information about the error.
    pub annotations: Vec<String>,
}

/// The kind of parser error.
#[derive(Debug, Eq, PartialEq)]
pub enum ParserErrorKind {
    /// Found a second root section, but only one is allowed.
    SecondRootSection {
        /// The label of the second root section.
        label: String,
    },

    /// Found a section at a level that is deeper than supported.
    SectionLevelTooDeep {
        /// The too deep level.
        level: usize,
    },

    /// Found a double close brace that does not match any opened one.
    UnmatchedDoubleCloseBrace,

    /// Found a double open brace that does not match any closed one.
    UnmatchedDoubleOpenBrace,

    /// Found a double close bracket that does not match any opened one.
    UnmatchedDoubleCloseBracket,

    /// Found a double open bracket that does not match any closed one.
    UnmatchedDoubleOpenBracket,

    /// Found a `</nowiki>` that does not match any `<nowiki>`.
    UnmatchedNoWikiClose,

    /// Found a `<nowiki>` that does not match any `</nowiki>`.
    UnmatchedNoWikiOpen,

    /// A tag contains a token that does not belong there.
    UnexpectedTokenInTag {
        /// The unexpected token.
        token: String,
    },

    /// A parameter contains a token that does not belong there.
    UnexpectedTokenInParameter {
        /// The unexpected token.
        token: String,
    },

    /// A link contains a token that does not belong there.
    UnexpectedTokenInLink {
        /// The unexpected token.
        token: String,
    },

    /// A link label contains a token that does not belong there.
    UnexpectedTokenInLinkLabel {
        /// The unexpected token.
        token: String,
    },

    /// A formatted piece of text contains a token that does not belong there.
    UnexpectedTokenInFormattedText {
        /// The unexpected token.
        token: String,
    },

    /// A link label contains a token that does not belong there.
    UnexpectedTokenInListItem {
        /// The unexpected token.
        token: String,
    },

    /// A token was found at a place where it does not belong.
    UnexpectedToken {
        /// The expected token, or a list of possible expected tokens.
        expected: String,
        /// The token that was found.
        actual: String,
    },

    /// A text formatting expression was not closed.
    UnclosedTextFormatting {
        /// The unclosed formatting expression.
        formatting: TextFormatting,
    },

    /// The end of file was found, but further tokens were expected.
    UnexpectedEof,
}

impl Display for ParserErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParserErrorKind::SecondRootSection { label } => {
                write!(f, "found second root section {label:?}")
            }
            ParserErrorKind::SectionLevelTooDeep { level } => {
                write!(f, "found a section of a too deep level {level}")
            }
            ParserErrorKind::UnmatchedDoubleCloseBrace => {
                write!(f, "found an unmatched double closing brace }}}}")
            }
            ParserErrorKind::UnmatchedDoubleOpenBrace => {
                write!(f, "found an unmatched double open brace {{{{")
            }
            ParserErrorKind::UnmatchedDoubleCloseBracket => {
                write!(f, "found an unmatched double close bracket ]]")
            }
            ParserErrorKind::UnmatchedDoubleOpenBracket => {
                write!(f, "found an unmatched double open bracket [[")
            }
            ParserErrorKind::UnmatchedNoWikiClose => {
                write!(f, "found an unmatched nowiki close tag </nowiki>")
            }
            ParserErrorKind::UnmatchedNoWikiOpen => {
                write!(f, "found an unmatched nowiki open tag <nowiki>")
            }
            ParserErrorKind::UnexpectedTokenInTag { token } => {
                write!(f, "found an unexpected token {token:?} in a tag")
            }
            ParserErrorKind::UnexpectedTokenInParameter { token } => {
                write!(f, "found an unexpected token {token:?} in a parameter")
            }
            ParserErrorKind::UnexpectedTokenInLink { token } => {
                write!(f, "found an unexpected token {token:?} in a link")
            }
            ParserErrorKind::UnexpectedTokenInLinkLabel { token } => {
                write!(f, "found an unexpected token {token:?} in a link label")
            }
            ParserErrorKind::UnexpectedTokenInFormattedText { token } => {
                write!(f, "found an unexpected token {token:?} in formatted text")
            }
            ParserErrorKind::UnexpectedTokenInListItem { token } => {
                write!(f, "found an unexpected token {token:?} in a list item")
            }
            ParserErrorKind::UnexpectedToken { expected, actual } => write!(
                f,
                "found an unexpected token {actual:?} where {expected:?} was expected"
            ),
            ParserErrorKind::UnclosedTextFormatting { formatting } => write!(
                f,
                "found an unclosed text formatting expression {formatting}:?"
            ),
            ParserErrorKind::UnexpectedEof => {
                write!(f, "the file ended, but we expected more content")
            }
        }
    }
}

impl Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} at line {}, column {}",
            self.kind, self.position.line, self.position.column
        )?;

        if !self.annotations.is_empty() {
            write!(f, "; additional information: [")?;
            let mut once = true;
            for annotation in &self.annotations {
                if once {
                    once = false;
                } else {
                    write!(f, ", ")?;
                }
                write!(f, "{annotation}")?;
            }
            write!(f, "]")?;
        }

        Ok(())
    }
}

impl Error for ParserError {}

impl ParserError {
    /// Add the given annotation to the error.
    pub fn annotate(&mut self, annotation: String) {
        self.annotations.push(annotation);
    }

    /// Add the given annotation to the error.
    pub fn annotate_self(mut self, annotation: String) -> Self {
        self.annotations.push(annotation);
        self
    }
}

impl ParserErrorKind {
    pub(crate) fn into_parser_error(self, position: TextPosition) -> ParserError {
        ParserError {
            kind: self,
            position,
            annotations: Default::default(),
        }
    }
}
