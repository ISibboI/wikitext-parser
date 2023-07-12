use crate::error::{ParserErrorKind, Result};
use crate::level_stack::LevelStack;
use crate::tokenizer::{MultipeekTokenizer, Token, Tokenizer};
use crate::wikitext::{Attribute, Headline, Line, Text, TextFormatting, TextPiece, Wikitext};
use std::mem;

#[cfg(not(test))]
static DO_PARSER_DEBUG_PRINTS: bool = false;
#[cfg(test)]
static DO_PARSER_DEBUG_PRINTS: bool = true;

/// Parse textual wikitext into a semantic representation.
pub fn parse_wikitext(wikitext: &str, headline: String) -> Result<Wikitext> {
    let mut level_stack = LevelStack::new(headline);
    let mut tokenizer = MultipeekTokenizer::new(Tokenizer::new(wikitext));

    loop {
        tokenizer.peek(1);
        if DO_PARSER_DEBUG_PRINTS {
            println!(
                "parse_wikitext tokens: {:?} {:?}",
                tokenizer.repeek(0),
                tokenizer.repeek(1),
            );
        }

        if tokenizer.repeek(0).unwrap().0 == Token::Newline
            && tokenizer.repeek(1).unwrap().0 == Token::Newline
        {
            level_stack.new_paragraph();
            tokenizer.next();
            continue;
        }

        let (token, _) = tokenizer.peek(0);

        if matches!(token, Token::MultiEquals(_)) {
            if let Some(headline) = parse_potential_headline(&mut tokenizer) {
                level_stack.append_headline(headline?);
                continue;
            }
        } else if token == &Token::Eof {
            break;
        }

        level_stack.append_line(parse_line(&mut tokenizer)?);
    }

    Ok(Wikitext {
        root_section: level_stack.into_root_section(),
    })
}

fn parse_line(tokenizer: &mut MultipeekTokenizer) -> Result<Line> {
    debug_assert!(parse_potential_headline(tokenizer).is_none());

    let mut list_prefix = String::new();

    // parse list_prefix
    while let token @ (Token::Colon | Token::Semicolon | Token::Star | Token::Sharp) =
        &tokenizer.peek(0).0
    {
        list_prefix.push_str(token.to_str());
        tokenizer.next();
    }

    // parse remaining text
    if !list_prefix.is_empty() {
        let text = parse_text_until(tokenizer, Text::new(), TextFormatting::Normal, |token| {
            matches!(token, Token::Newline | Token::Eof)
        })?;
        tokenizer.next();
        Ok(Line::List { list_prefix, text })
    } else {
        let text = parse_text_until(tokenizer, Text::new(), TextFormatting::Normal, |token| {
            matches!(token, Token::Newline | Token::Eof)
        })?;
        tokenizer.next();
        Ok(Line::Normal { text })
    }
}

fn parse_text_until(
    tokenizer: &mut MultipeekTokenizer,
    mut prefix: Text,
    mut text_formatting: TextFormatting,
    terminator: impl Fn(&Token<'_>) -> bool,
) -> Result<Text> {
    loop {
        if DO_PARSER_DEBUG_PRINTS {
            println!("parse_text_until token: {:?}", tokenizer.peek(0));
        }
        let (token, text_position) = tokenizer.peek(0);
        if terminator(token) {
            break;
        }

        match token {
            token @ (Token::Text(_)
            | Token::MultiEquals(_)
            | Token::Colon
            | Token::Semicolon
            | Token::Star
            | Token::Sharp
            | Token::Newline
            | Token::VerticalBar) => {
                prefix.extend_with_formatted_text(text_formatting, token.to_str());
                tokenizer.next();
            }
            Token::DoubleOpenBrace => prefix
                .pieces
                .push(parse_double_brace_expression(tokenizer, text_formatting)?),
            Token::DoubleOpenBracket => prefix
                .pieces
                .push(parse_internal_link(tokenizer, text_formatting)?),
            Token::DoubleCloseBrace => {
                return Err(
                    ParserErrorKind::UnmatchedDoubleCloseBrace.into_parser_error(*text_position)
                )
            }
            Token::DoubleCloseBracket => {
                return Err(
                    ParserErrorKind::UnmatchedDoubleCloseBracket.into_parser_error(*text_position)
                )
            }
            Token::Apostrophe => {
                tokenizer.peek(4);
                let apostrophe_prefix_length = (0..5)
                    .take_while(|i| tokenizer.peek(*i).0 == Token::Apostrophe)
                    .count();
                if apostrophe_prefix_length == 1 {
                    prefix.extend_with_formatted_text(text_formatting, "'");
                    tokenizer.next();
                } else {
                    let apostrophe_prefix_length = if apostrophe_prefix_length == 4 {
                        3
                    } else {
                        apostrophe_prefix_length
                    };
                    text_formatting = text_formatting.next_formatting(apostrophe_prefix_length);
                    for _ in 0..apostrophe_prefix_length {
                        tokenizer.next();
                    }
                }
            }
            Token::Eof => {
                return Err(ParserErrorKind::UnexpectedEof.into_parser_error(*text_position))
            }
        }
    }

    Ok(prefix)
}

fn parse_potential_headline(tokenizer: &mut MultipeekTokenizer) -> Option<Result<Headline>> {
    tokenizer.peek(4);
    if let (
        Some((Token::MultiEquals(first_level), text_position)),
        Some((Token::Text(_), _)),
        Some((Token::MultiEquals(second_level), _)),
    ) = (
        tokenizer.repeek(0),
        tokenizer.repeek(1),
        tokenizer.repeek(2),
    ) {
        let text_position = *text_position;
        if first_level == second_level {
            let level = *first_level;
            let suffix = if matches!(tokenizer.repeek(3), Some((Token::Newline | Token::Eof, _))) {
                Some(1)
            } else if matches!(tokenizer.repeek(4), Some((Token::Newline | Token::Eof, _))) {
                if let (Token::Text(text), _) = tokenizer.repeek(3).unwrap() {
                    if text.chars().all(|c| c.is_ascii_whitespace() && c != '\n') {
                        Some(2)
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            };

            if let Some(suffix) = suffix {
                let Some((Token::Text(text), _)) = tokenizer.repeek(1) else { unreachable!("Tokenizer was not mutated after matching the same repeek above.") };
                debug_assert!(!text.contains('\n'));
                let label = text.trim().to_string();

                if level == 1 {
                    Some(Err(ParserErrorKind::SecondRootSection { label }
                        .into_parser_error(text_position)))
                } else {
                    tokenizer.next();
                    tokenizer.next();
                    tokenizer.next();
                    for _ in 0..suffix {
                        tokenizer.next();
                    }
                    Some(Ok(Headline { label, level }))
                }
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    }
}

fn parse_double_brace_expression(
    tokenizer: &mut MultipeekTokenizer,
    text_formatting: TextFormatting,
) -> Result<TextPiece> {
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
        let (token, text_position) = tokenizer.peek(0);
        match token {
            Token::VerticalBar => attributes.push(parse_attribute(tokenizer, text_formatting)?),
            Token::DoubleCloseBrace => {
                tokenizer.next();
                break;
            }
            token @ (Token::Text(_)
            | Token::MultiEquals(_)
            | Token::DoubleOpenBrace
            | Token::DoubleOpenBracket
            | Token::DoubleCloseBracket
            | Token::Apostrophe
            | Token::Newline
            | Token::Colon
            | Token::Semicolon
            | Token::Star
            | Token::Sharp) => {
                return Err(ParserErrorKind::UnexpectedToken {
                    expected: "| or }}".to_string(),
                    actual: token.to_string(),
                }
                .into_parser_error(*text_position))
            }
            Token::Eof => {
                return Err(
                    ParserErrorKind::UnmatchedDoubleOpenBrace.into_parser_error(*text_position)
                )
            }
        }
    }

    Ok(TextPiece::DoubleBraceExpression { tag, attributes })
}

fn parse_tag(tokenizer: &mut MultipeekTokenizer) -> Result<String> {
    let mut tag = String::new();

    loop {
        let (token, text_position) = tokenizer.peek(0);
        match token {
            Token::Text(text) => tag.push_str(text),
            token @ (Token::Newline | Token::Colon) => tag.push_str(token.to_str()),
            Token::DoubleCloseBrace | Token::VerticalBar => break,
            Token::Eof => {
                return Err(
                    ParserErrorKind::UnmatchedDoubleOpenBrace.into_parser_error(*text_position)
                )
            }
            token @ (Token::MultiEquals(_)
            | Token::DoubleOpenBrace
            | Token::DoubleOpenBracket
            | Token::DoubleCloseBracket
            | Token::Apostrophe
            | Token::Semicolon
            | Token::Star
            | Token::Sharp) => {
                return Err(ParserErrorKind::UnexpectedTokenInTag {
                    token: token.to_string(),
                }
                .into_parser_error(*text_position))
            }
        }

        tokenizer.next();
    }

    tag = tag.trim().to_string();
    Ok(tag)
}

fn parse_attribute(
    tokenizer: &mut MultipeekTokenizer,
    text_formatting: TextFormatting,
) -> Result<Attribute> {
    tokenizer.expect(&Token::VerticalBar)?;
    let mut name = Some(String::new());
    let mut value = Text::new();

    // parse name
    loop {
        if DO_PARSER_DEBUG_PRINTS {
            println!("parse_attribute name token: {:?}", tokenizer.peek(0));
        }
        let (token, text_position) = tokenizer.peek(0);
        match token {
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
            | Token::Apostrophe
            | Token::Colon
            | Token::Semicolon
            | Token::Star
            | Token::Sharp => {
                value.pieces.push(TextPiece::Text {
                    text: name.take().unwrap(),
                    formatting: text_formatting,
                });
                break;
            }
            token @ (Token::MultiEquals(_) | Token::DoubleCloseBracket) => {
                return Err(ParserErrorKind::UnexpectedTokenInParameter {
                    token: token.to_string(),
                }
                .into_parser_error(*text_position))
            }
            Token::Eof => {
                return Err(
                    ParserErrorKind::UnmatchedDoubleOpenBrace.into_parser_error(*text_position)
                )
            }
        }
    }

    // parse value
    let mut value = parse_text_until(tokenizer, value, text_formatting, |token| {
        matches!(token, Token::VerticalBar | Token::DoubleCloseBrace)
    })
    .map_err(|error| error.annotate_self("parse_double_brace_expression value".to_string()))?;

    // whitespace is stripped from named attribute names and values, but not from unnamed attributes
    if let Some(name) = &mut name {
        *name = name.trim().to_string();
        value.trim_self();
    }

    Ok(Attribute { name, value })
}

fn parse_internal_link(
    tokenizer: &mut MultipeekTokenizer,
    text_formatting: TextFormatting,
) -> Result<TextPiece> {
    tokenizer.expect(&Token::DoubleOpenBracket)?;
    let mut target = String::new();
    let mut options = Vec::new();
    let mut label = None;

    // parse target
    loop {
        if DO_PARSER_DEBUG_PRINTS {
            println!("parse_link target token: {:?}", tokenizer.peek(0));
        }
        let (token, text_position) = tokenizer.peek(0);
        match token {
            token @ (Token::Text(_)
            | Token::Colon
            | Token::Sharp
            | Token::Semicolon
            | Token::Star) => {
                target.push_str(token.to_str());
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
            | Token::Apostrophe
            | Token::Newline) => {
                return Err(ParserErrorKind::UnexpectedTokenInLink {
                    token: token.to_string(),
                }
                .into_parser_error(*text_position))
            }
            Token::Eof => {
                return Err(
                    ParserErrorKind::UnmatchedDoubleOpenBracket.into_parser_error(*text_position)
                )
            }
        }
    }

    // parse options and label
    let label = label.map(|mut label| {
        let mut link_finished = false;

        // parse options
        loop {
            if DO_PARSER_DEBUG_PRINTS {
                println!("parse_link options token: {:?}", tokenizer.peek(0));
            }
            let (token, text_position) = tokenizer.peek(0);
            match token {
                Token::Text(text) => {
                    label.extend_with_formatted_text(text_formatting, text);
                    tokenizer.next();
                }
                Token::VerticalBar => {
                    let mut new_label = Text::new();
                    mem::swap(&mut label, &mut new_label);
                    assert_eq!(new_label.pieces.len(), 1);
                    let TextPiece::Text { text, ..}= new_label.pieces.into_iter().next().unwrap() else {
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
                Token::DoubleOpenBrace
                | Token::DoubleOpenBracket
                | Token::Apostrophe
                | Token::Colon
                | Token::Semicolon
                | Token::Star
                | Token::Sharp => {
                    break;
                }
                token @ (Token::MultiEquals(_) | Token::DoubleCloseBrace | Token::Newline) => {
                    return Err(ParserErrorKind::UnexpectedTokenInLinkLabel {
                        token: token.to_string(),
                    }
                        .into_parser_error(*text_position))
                }
                Token::Eof => {
                    return Err(ParserErrorKind::UnmatchedDoubleOpenBracket
                        .into_parser_error(*text_position))
                }
            }
        }

        Ok(if !link_finished {
            // parse label
            let label = parse_text_until(tokenizer, label, text_formatting, |token| matches!(token, Token::DoubleCloseBracket)).map_err(|error| error.annotate_self("parse_internal_link label".to_string()))?;
            let next_token = tokenizer.expect(&Token::DoubleCloseBracket);
            debug_assert!(next_token.is_ok());
            label
        } else {
            label
        })
    }).transpose()?;

    Ok(TextPiece::InternalLink {
        target,
        options,
        label,
    })
}