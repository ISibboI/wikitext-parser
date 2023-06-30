use crate::error::{ParserErrorKind, Result};
use crate::level_stack::LevelStack;
use crate::tokenizer::{MultipeekTokenizer, Token, Tokenizer};
use crate::wikitext::{Attribute, Headline, Text, TextPiece, Wikitext};

static DO_PARSER_DEBUG_PRINTS: bool = false;

/// Parse textual wikitext into a semantic representation.
pub fn parse_wikitext(wikitext: &str, headline: String) -> Result<Wikitext> {
    let mut level_stack = LevelStack::new(headline);
    let mut tokenizer = MultipeekTokenizer::new(Tokenizer::new(wikitext));

    loop {
        if DO_PARSER_DEBUG_PRINTS {
            println!("parse_wikitext token: {:?}", tokenizer.peek(0));
        }
        match tokenizer.peek(0) {
            Token::Text(text) => {
                level_stack.append_text_piece(TextPiece::Text(text.to_string()));
                tokenizer.next();
            }
            Token::MultiEquals(count) => {
                let count = *count;
                if let Some(headline) = parse_potential_headline(&mut tokenizer, count) {
                    if headline.level == 1 {
                        return Err(ParserErrorKind::SecondRootSection {
                            label: headline.label,
                        }
                        .into_parser_error(tokenizer.text_position()));
                    }
                    level_stack.append_headline(headline);
                } else {
                    level_stack
                        .append_text_piece(TextPiece::Text(Token::MultiEquals(count).to_string()));
                    tokenizer.next();
                }
            }
            Token::DoubleOpenBrace => {
                level_stack.append_text_piece(parse_double_brace_expression(&mut tokenizer)?)
            }
            Token::DoubleCloseBrace => {
                return Err(ParserErrorKind::UnmatchedDoubleCloseBrace
                    .into_parser_error(tokenizer.text_position()))
            }
            Token::Eof => break,
            ignored_token => {
                level_stack.extend_text_piece(ignored_token.to_str());
                tokenizer.next();
            }
        }
    }

    Ok(Wikitext {
        root_section: level_stack.into_root_section(),
    })
}

fn parse_potential_headline(tokenizer: &mut MultipeekTokenizer, level: u8) -> Option<Headline> {
    tokenizer.peek(2);
    if let (
        Some(Token::MultiEquals(first_level)),
        Some(Token::Text(text)),
        Some(Token::MultiEquals(second_level)),
    ) = (
        tokenizer.repeek(0),
        tokenizer.repeek(1),
        tokenizer.repeek(2),
    ) {
        if level == *first_level && level == *second_level && !text.contains('\n') {
            let label = text.trim().to_string();
            tokenizer.next();
            tokenizer.next();
            tokenizer.next();
            Some(Headline { label, level })
        } else {
            None
        }
    } else {
        None
    }
}

fn parse_double_brace_expression(tokenizer: &mut MultipeekTokenizer) -> Result<TextPiece> {
    tokenizer.expect(&Token::DoubleOpenBrace)?;
    let tag = parse_tag(tokenizer)?;
    let mut attributes = Vec::new();

    loop {
        if DO_PARSER_DEBUG_PRINTS {
            println!(
                "parse_double_brace_expression token: {:?}",
                tokenizer.peek(0)
            );
        }
        match tokenizer.peek(0) {
            Token::VerticalBar => attributes.push(parse_attribute(tokenizer)?),
            Token::DoubleCloseBrace => {
                tokenizer.next();
                break;
            }
            token @ (Token::Text(_) | Token::MultiEquals(_) | Token::DoubleOpenBrace) => {
                return Err(ParserErrorKind::UnexpectedToken {
                    expected: "| or }}".to_string(),
                    actual: token.to_string(),
                }
                .into_parser_error(tokenizer.text_position()))
            }
            Token::Eof => {
                return Err(ParserErrorKind::UnmatchedDoubleOpenBrace
                    .into_parser_error(tokenizer.text_position()))
            }
        }
    }

    Ok(TextPiece::DoubleBraceExpression { tag, attributes })
}

fn parse_tag(tokenizer: &mut MultipeekTokenizer) -> Result<String> {
    let mut tag = String::new();

    loop {
        match tokenizer.peek(0) {
            Token::Text(text) => tag.push_str(text),
            Token::DoubleCloseBrace | Token::VerticalBar => break,
            Token::Eof => {
                return Err(ParserErrorKind::UnmatchedDoubleOpenBrace
                    .into_parser_error(tokenizer.text_position()))
            }
            token @ (Token::MultiEquals(_) | Token::DoubleOpenBrace) => {
                return Err(ParserErrorKind::UnexpectedTokenInTag {
                    token: token.to_string(),
                }
                .into_parser_error(tokenizer.text_position()))
            }
        }

        tokenizer.next();
    }

    Ok(tag)
}

fn parse_attribute(tokenizer: &mut MultipeekTokenizer) -> Result<Attribute> {
    tokenizer.expect(&Token::VerticalBar)?;
    let mut name = Some(String::new());
    let mut value = Text::new();

    // parse name
    loop {
        if DO_PARSER_DEBUG_PRINTS {
            println!("parse_attribute name token: {:?}", tokenizer.peek(0));
        }
        match tokenizer.peek(0) {
            Token::Text(text) => {
                name.iter_mut().next().unwrap().push_str(text);
                tokenizer.next();
            }
            Token::MultiEquals(1) => {
                tokenizer.next();
                break;
            }
            Token::DoubleOpenBrace | Token::VerticalBar | Token::DoubleCloseBrace => {
                value.pieces.push(TextPiece::Text(name.take().unwrap()));
                break;
            }
            token @ Token::MultiEquals(_) => {
                return Err(ParserErrorKind::UnexpectedTokenInParameter {
                    token: token.to_string(),
                }
                .into_parser_error(tokenizer.text_position()))
            }
            Token::Eof => {
                return Err(ParserErrorKind::UnmatchedDoubleOpenBrace
                    .into_parser_error(tokenizer.text_position()))
            }
        }
    }

    // parse value
    loop {
        if DO_PARSER_DEBUG_PRINTS {
            println!("parse_attribute value token: {:?}", tokenizer.peek(0));
        }
        match tokenizer.peek(0) {
            Token::Text(text) => {
                value.extend_with_text(text);
                tokenizer.next();
            }
            Token::DoubleOpenBrace => {
                value.pieces.push(parse_double_brace_expression(tokenizer)?);
            }
            Token::VerticalBar | Token::DoubleCloseBrace => break,
            token @ Token::MultiEquals(_) => {
                return Err(ParserErrorKind::UnexpectedTokenInParameter {
                    token: token.to_string(),
                }
                .into_parser_error(tokenizer.text_position()))
            }
            Token::Eof => {
                return Err(ParserErrorKind::UnmatchedDoubleOpenBrace
                    .into_parser_error(tokenizer.text_position()))
            }
        }
    }

    Ok(Attribute { name, value })
}
