use crate::error::{ParserErrorKind, Result};
use crate::level_stack::LevelStack;
use crate::tokenizer::{MultipeekTokenizer, Token, Tokenizer};
use crate::wikitext::{Attribute, Headline, Text, TextFormatting, TextPiece, Wikitext};
use std::mem;

#[cfg(not(tests))]
static DO_PARSER_DEBUG_PRINTS: bool = false;
#[cfg(tests)]
static DO_PARSER_DEBUG_PRINTS: bool = true;

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
            Token::DoubleOpenBracket => level_stack.append_text_piece(parse_link(&mut tokenizer)?),
            Token::DoubleCloseBrace => {
                return Err(ParserErrorKind::UnmatchedDoubleCloseBrace
                    .into_parser_error(tokenizer.text_position()))
            }
            Token::DoubleCloseBracket => {
                return Err(ParserErrorKind::UnmatchedDoubleCloseBracket
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
            token @ (Token::Text(_)
            | Token::MultiEquals(_)
            | Token::DoubleOpenBrace
            | Token::DoubleOpenBracket
            | Token::DoubleCloseBracket
            | Token::DoubleApostrophe
            | Token::TripleApostrophe
            | Token::QuintupleApostrophe
            | Token::Newline) => {
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
            Token::Newline => tag.push('\''),
            Token::DoubleCloseBrace | Token::VerticalBar => break,
            Token::Eof => {
                return Err(ParserErrorKind::UnmatchedDoubleOpenBrace
                    .into_parser_error(tokenizer.text_position()))
            }
            token @ (Token::MultiEquals(_)
            | Token::DoubleOpenBrace
            | Token::DoubleOpenBracket
            | Token::DoubleCloseBracket
            | Token::DoubleApostrophe
            | Token::TripleApostrophe
            | Token::QuintupleApostrophe) => {
                return Err(ParserErrorKind::UnexpectedTokenInTag {
                    token: token.to_string(),
                }
                .into_parser_error(tokenizer.text_position()))
            }
        }

        tokenizer.next();
    }

    tag = tag.trim().to_string();
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
            Token::Newline => {
                name.iter_mut().next().unwrap().push('\'');
                tokenizer.next();
            }
            Token::MultiEquals(1) => {
                tokenizer.next();
                break;
            }
            Token::DoubleOpenBrace
            | Token::DoubleOpenBracket
            | Token::VerticalBar
            | Token::DoubleCloseBrace
            | Token::DoubleApostrophe
            | Token::TripleApostrophe
            | Token::QuintupleApostrophe => {
                value.pieces.push(TextPiece::Text(name.take().unwrap()));
                break;
            }
            token @ (Token::MultiEquals(_) | Token::DoubleCloseBracket) => {
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
            token @ (Token::MultiEquals(_) | Token::Newline) => {
                value.extend_with_text(token.to_str());
                tokenizer.next();
            }
            Token::DoubleOpenBrace => value.pieces.push(parse_double_brace_expression(tokenizer)?),
            Token::DoubleOpenBracket => value.pieces.push(parse_link(tokenizer)?),
            token @ (Token::DoubleApostrophe
            | Token::TripleApostrophe
            | Token::QuintupleApostrophe) => {
                let text_formatting = token.as_text_formatting();
                value
                    .pieces
                    .push(parse_formatted_text(tokenizer, text_formatting)?);
            }
            Token::VerticalBar | Token::DoubleCloseBrace => break,
            token @ Token::DoubleCloseBracket => {
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

    // whitespace is stripped from named attribute names and values, but not from unnamend attributes
    if let Some(name) = &mut name {
        *name = name.trim().to_string();
        value.trim_self();
    }

    Ok(Attribute { name, value })
}

fn parse_link(tokenizer: &mut MultipeekTokenizer) -> Result<TextPiece> {
    tokenizer.expect(&Token::DoubleOpenBracket)?;
    let mut url = String::new();
    let mut options = Vec::new();
    let mut label = None;

    // parse url
    loop {
        if DO_PARSER_DEBUG_PRINTS {
            println!("parse_link url token: {:?}", tokenizer.peek(0));
        }
        match tokenizer.peek(0) {
            Token::Text(text) => {
                url.push_str(text);
                tokenizer.next();
            }
            Token::DoubleCloseBracket => {
                tokenizer.next();
                break;
            }
            Token::VerticalBar => {
                tokenizer.next();
                label = Some(Text::new());
                break;
            }
            token @ (Token::MultiEquals(_)
            | Token::DoubleOpenBrace
            | Token::DoubleCloseBrace
            | Token::DoubleOpenBracket
            | Token::DoubleApostrophe
            | Token::TripleApostrophe
            | Token::QuintupleApostrophe
            | Token::Newline) => {
                return Err(ParserErrorKind::UnexpectedTokenInLink {
                    token: token.to_string(),
                }
                .into_parser_error(tokenizer.text_position()))
            }
            Token::Eof => {
                return Err(ParserErrorKind::UnmatchedDoubleOpenBracket
                    .into_parser_error(tokenizer.text_position()))
            }
        }
    }

    // parse options and label
    if let Some(label) = label.as_mut() {
        let mut link_finished = false;

        // parse options
        loop {
            if DO_PARSER_DEBUG_PRINTS {
                println!("parse_link options token: {:?}", tokenizer.peek(0));
            }
            match tokenizer.peek(0) {
                Token::Text(text) => {
                    label.extend_with_text(text);
                    tokenizer.next();
                }
                Token::VerticalBar => {
                    let mut new_label = Text::new();
                    mem::swap(label, &mut new_label);
                    assert_eq!(new_label.pieces.len(), 1);
                    let TextPiece::Text(text) = new_label.pieces.into_iter().next().unwrap() else {
                        unreachable!("Only text is ever inserted into link options");
                    };
                    options.push(text);
                    tokenizer.next();
                }
                Token::DoubleCloseBracket => {
                    tokenizer.next();
                    link_finished = true;
                    break;
                }
                Token::DoubleApostrophe | Token::TripleApostrophe | Token::QuintupleApostrophe => {
                    break;
                }
                token @ (Token::MultiEquals(_)
                | Token::DoubleOpenBrace
                | Token::DoubleCloseBrace
                | Token::DoubleOpenBracket
                | Token::Newline) => {
                    return Err(ParserErrorKind::UnexpectedTokenInLinkLabel {
                        token: token.to_string(),
                    }
                    .into_parser_error(tokenizer.text_position()))
                }
                Token::Eof => {
                    return Err(ParserErrorKind::UnmatchedDoubleOpenBracket
                        .into_parser_error(tokenizer.text_position()))
                }
            }
        }

        if !link_finished {
            // parse label
            loop {
                if DO_PARSER_DEBUG_PRINTS {
                    println!("parse_link label token: {:?}", tokenizer.peek(0));
                }
                match tokenizer.peek(0) {
                    Token::Text(text) => {
                        label.extend_with_text(text);
                        tokenizer.next();
                    }
                    token @ (Token::DoubleApostrophe
                    | Token::TripleApostrophe
                    | Token::QuintupleApostrophe) => {
                        let text_formatting = token.as_text_formatting();
                        label
                            .pieces
                            .push(parse_formatted_text(tokenizer, text_formatting)?);
                    }
                    Token::DoubleCloseBracket => {
                        tokenizer.next();
                        break;
                    }
                    token @ (Token::MultiEquals(_)
                    | Token::DoubleOpenBrace
                    | Token::DoubleCloseBrace
                    | Token::DoubleOpenBracket
                    | Token::VerticalBar
                    | Token::Newline) => {
                        return Err(ParserErrorKind::UnexpectedTokenInLinkLabel {
                            token: token.to_string(),
                        }
                        .into_parser_error(tokenizer.text_position()))
                    }
                    Token::Eof => {
                        return Err(ParserErrorKind::UnmatchedDoubleOpenBracket
                            .into_parser_error(tokenizer.text_position()))
                    }
                }
            }
        }
    }

    Ok(TextPiece::Link {
        url,
        options,
        label,
    })
}

fn parse_formatted_text(
    tokenizer: &mut MultipeekTokenizer,
    text_formatting: TextFormatting,
) -> Result<TextPiece> {
    let mut text = Text::new();
    tokenizer.expect(&Token::from(text_formatting))?;

    loop {
        if DO_PARSER_DEBUG_PRINTS {
            println!("parse_formatted_text token: {:?}", tokenizer.peek(0));
        }

        if tokenizer.peek(0) == &Token::from(text_formatting) {
            tokenizer.next();
            break;
        }

        match tokenizer.peek(0) {
            Token::Text(new_text) => {
                text.extend_with_text(new_text);
                tokenizer.next();
            }
            token @ Token::MultiEquals(1) => {
                text.extend_with_text(token.to_str());
                tokenizer.next();
            }
            token @ (Token::MultiEquals(_)
            | Token::Newline
            | Token::DoubleCloseBrace
            | Token::DoubleCloseBracket
            | Token::VerticalBar
            | Token::DoubleApostrophe
            | Token::TripleApostrophe
            | Token::QuintupleApostrophe) => {
                return Err(ParserErrorKind::UnexpectedTokenInFormattedText {
                    token: token.to_string(),
                }
                .into_parser_error(tokenizer.text_position()))
            }
            Token::Eof => {
                return Err(ParserErrorKind::UnclosedTextFormatting {
                    formatting: text_formatting,
                }
                .into_parser_error(tokenizer.text_position()))
            }
            Token::DoubleOpenBrace => text.pieces.push(parse_double_brace_expression(tokenizer)?),
            Token::DoubleOpenBracket => text.pieces.push(parse_link(tokenizer)?),
        }
    }

    Ok(TextPiece::FormattedText {
        text,
        formatting: text_formatting,
    })
}
