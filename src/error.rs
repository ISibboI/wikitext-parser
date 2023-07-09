use crate::tokenizer::TextPosition;
use crate::wikitext::TextFormatting;

pub type Result<T> = std::result::Result<T, ParserError>;

/// Error type of this crate.
#[derive(Debug)]
pub struct ParserError {
    /// The kind of error.
    pub kind: ParserErrorKind,
    /// The position of the error in text.
    pub position: TextPosition,
}

/// The kind of parser error.
#[derive(Debug)]
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
}

impl ParserErrorKind {
    pub(crate) fn into_parser_error(self, position: TextPosition) -> ParserError {
        ParserError {
            kind: self,
            position,
        }
    }
}
