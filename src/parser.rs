use crate::error::{ParserErrorKind, Result};
use crate::level_stack::LevelStack;
use crate::tokenizer::{MultipeekTokenizer, Token, Tokenizer};
use crate::wikitext::{Attribute, Headline, Text, TextFormatting, TextPiece, Wikitext};
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
        if DO_PARSER_DEBUG_PRINTS {
            println!("parse_wikitext token: {:?}", tokenizer.peek(0));
        }
        let (token, text_position) = tokenizer.peek(0);
        match token {
            token @ Token::Text(_) => {
                level_stack.extend_text_piece(token.to_str());
                tokenizer.next();
            }
            Token::MultiEquals(amount) => {
                let amount = *amount;
                let is_headline = if tokenizer.is_at_start() {
                    if let Some(headline) = parse_potential_headline(&mut tokenizer) {
                        level_stack.append_headline(headline?);
                        true
                    } else {
                        false
                    }
                } else {
                    false
                };

                if !is_headline {
                    level_stack.extend_text_piece(Token::MultiEquals(amount).to_str());
                    tokenizer.next();
                }
            }
            Token::DoubleOpenBrace => {
                level_stack.append_text_piece(parse_double_brace_expression(&mut tokenizer)?)
            }
            Token::DoubleOpenBracket => {
                level_stack.append_text_piece(parse_internal_link(&mut tokenizer)?)
            }
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
            token @ (Token::Colon | Token::Semicolon | Token::Star | Token::Sharp) => {
                let token = match token {
                    Token::Colon => Token::Colon,
                    Token::Semicolon => Token::Semicolon,
                    Token::Star => Token::Star,
                    Token::Sharp => Token::Sharp,
                    other => unreachable!("token {other} is not matched by parent pattern"),
                };
                let is_list_item = if tokenizer.is_at_start() {
                    if let Some(list_item) = parse_potential_list_item(&mut tokenizer) {
                        level_stack.append_text_piece(list_item?);
                        true
                    } else {
                        false
                    }
                } else {
                    false
                };

                if !is_list_item {
                    level_stack.extend_text_piece(token.to_str());
                    tokenizer.next();
                }
            }
            Token::Newline => {
                if let Some(headline) = parse_potential_headline(&mut tokenizer) {
                    level_stack.append_headline(headline?);
                } else if let Some(list_item) = parse_potential_list_item(&mut tokenizer) {
                    level_stack.append_text_piece(list_item?);
                } else {
                    level_stack.extend_text_piece("\n");
                    tokenizer.next();
                }
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

fn parse_potential_headline(tokenizer: &mut MultipeekTokenizer) -> Option<Result<Headline>> {
    let offset = if !tokenizer.is_at_start() {
        if DO_PARSER_DEBUG_PRINTS {
            println!("parse_potential_headline is not at start");
        }
        assert_eq!(tokenizer.peek(0).0, Token::Newline, "parse_potential_headline should only be called at the beginning of the page or after a newline.");
        1
    } else {
        if DO_PARSER_DEBUG_PRINTS {
            println!("parse_potential_headline is at start");
        }
        0
    };

    tokenizer.peek(4 + offset);
    if let (
        Some((Token::MultiEquals(first_level), text_position)),
        Some((Token::Text(_), _)),
        Some((Token::MultiEquals(second_level), _)),
    ) = (
        tokenizer.repeek(offset),
        tokenizer.repeek(1 + offset),
        tokenizer.repeek(2 + offset),
    ) {
        let text_position = *text_position;
        if first_level == second_level {
            let level = *first_level;
            let suffix = if matches!(
                tokenizer.repeek(3 + offset),
                Some((Token::Newline | Token::Eof, _))
            ) {
                Some(1)
            } else if matches!(
                tokenizer.repeek(4 + offset),
                Some((Token::Newline | Token::Eof, _))
            ) {
                if let (Token::Text(text), _) = tokenizer.repeek(3 + offset).unwrap() {
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
                let Some((Token::Text(text), _)) = tokenizer.repeek(1 + offset) else { unreachable!("Tokenizer was not mutated after matching the same repeek above.") };
                debug_assert!(!text.contains('\n'));
                let label = text.trim().to_string();

                if level == 1 {
                    Some(Err(ParserErrorKind::SecondRootSection { label }
                        .into_parser_error(text_position)))
                } else {
                    tokenizer.next();
                    tokenizer.next();
                    tokenizer.next();
                    for _ in 0..offset + suffix {
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
        let (token, text_position) = tokenizer.peek(0);
        match token {
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
            | Token::DoubleApostrophe
            | Token::TripleApostrophe
            | Token::QuintupleApostrophe
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

fn parse_attribute(tokenizer: &mut MultipeekTokenizer) -> Result<Attribute> {
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
            | Token::DoubleApostrophe
            | Token::TripleApostrophe
            | Token::QuintupleApostrophe
            | Token::Colon
            | Token::Semicolon
            | Token::Star
            | Token::Sharp => {
                value.pieces.push(TextPiece::Text(name.take().unwrap()));
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
    loop {
        if DO_PARSER_DEBUG_PRINTS {
            println!("parse_attribute value token: {:?}", tokenizer.peek(0));
        }
        let (token, text_position) = tokenizer.peek(0);
        match token {
            Token::Text(text) => {
                value.extend_with_text(text);
                tokenizer.next();
            }
            token @ (Token::MultiEquals(_)
            | Token::Newline
            | Token::Colon
            | Token::Semicolon
            | Token::Star
            | Token::Sharp) => {
                value.extend_with_text(token.to_str());
                tokenizer.next();
            }
            Token::DoubleOpenBrace => value.pieces.push(parse_double_brace_expression(tokenizer)?),
            Token::DoubleOpenBracket => value.pieces.push(parse_internal_link(tokenizer)?),
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
                .into_parser_error(*text_position))
            }
            Token::Eof => {
                return Err(
                    ParserErrorKind::UnmatchedDoubleOpenBrace.into_parser_error(*text_position)
                )
            }
        }
    }

    // whitespace is stripped from named attribute names and values, but not from unnamed attributes
    if let Some(name) = &mut name {
        *name = name.trim().to_string();
        value.trim_self();
    }

    Ok(Attribute { name, value })
}

fn parse_internal_link(tokenizer: &mut MultipeekTokenizer) -> Result<TextPiece> {
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
            Token::Text(text) => {
                target.push_str(text);
                tokenizer.next();
            }
            token @ (Token::Colon | Token::Sharp | Token::Semicolon | Token::Star) => {
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
            | Token::DoubleApostrophe
            | Token::TripleApostrophe
            | Token::QuintupleApostrophe
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
    if let Some(label) = label.as_mut() {
        let mut link_finished = false;

        // parse options
        loop {
            if DO_PARSER_DEBUG_PRINTS {
                println!("parse_link options token: {:?}", tokenizer.peek(0));
            }
            let (token, text_position) = tokenizer.peek(0);
            match token {
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
                Token::DoubleOpenBrace
                | Token::DoubleOpenBracket
                | Token::DoubleApostrophe
                | Token::TripleApostrophe
                | Token::QuintupleApostrophe
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

        if !link_finished {
            // parse label
            loop {
                if DO_PARSER_DEBUG_PRINTS {
                    println!("parse_link label token: {:?}", tokenizer.peek(0));
                }
                let (token, text_position) = tokenizer.peek(0);
                match token {
                    Token::Text(text) => {
                        label.extend_with_text(text);
                        tokenizer.next();
                    }
                    Token::DoubleOpenBrace => {
                        label.pieces.push(parse_double_brace_expression(tokenizer)?)
                    }
                    Token::DoubleOpenBracket => label.pieces.push(parse_internal_link(tokenizer)?),
                    token @ (Token::DoubleApostrophe
                    | Token::TripleApostrophe
                    | Token::QuintupleApostrophe) => {
                        let text_formatting = token.as_text_formatting();
                        label
                            .pieces
                            .push(parse_formatted_text(tokenizer, text_formatting)?);
                    }
                    token @ (Token::Colon | Token::Semicolon | Token::Star | Token::Sharp) => {
                        label.extend_with_text(token.to_str());
                        tokenizer.next();
                    }
                    Token::DoubleCloseBracket => {
                        tokenizer.next();
                        break;
                    }
                    token @ (Token::MultiEquals(_)
                    | Token::DoubleCloseBrace
                    | Token::VerticalBar
                    | Token::Newline) => {
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
        }
    }

    Ok(TextPiece::InternalLink {
        target,
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

        if tokenizer.peek(0).0 == Token::from(text_formatting) {
            tokenizer.next();
            break;
        }

        let (token, text_position) = tokenizer.peek(0);
        match token {
            Token::Text(new_text) => {
                text.extend_with_text(new_text);
                tokenizer.next();
            }
            token @ (Token::MultiEquals(1)
            | Token::Colon
            | Token::Semicolon
            | Token::Star
            | Token::Sharp) => {
                text.extend_with_text(token.to_str());
                tokenizer.next();
            }
            Token::DoubleOpenBrace => text.pieces.push(parse_double_brace_expression(tokenizer)?),
            Token::DoubleOpenBracket => text.pieces.push(parse_internal_link(tokenizer)?),
            token @ (Token::DoubleApostrophe
            | Token::TripleApostrophe
            | Token::QuintupleApostrophe) => {
                let text_formatting = token.as_text_formatting();
                text.pieces
                    .push(parse_formatted_text(tokenizer, text_formatting)?);
            }
            token @ (Token::MultiEquals(_)
            | Token::Newline
            | Token::DoubleCloseBrace
            | Token::DoubleCloseBracket
            | Token::VerticalBar) => {
                return Err(ParserErrorKind::UnexpectedTokenInFormattedText {
                    token: token.to_string(),
                }
                .into_parser_error(*text_position))
            }
            Token::Eof => {
                return Err(ParserErrorKind::UnclosedTextFormatting {
                    formatting: text_formatting,
                }
                .into_parser_error(*text_position))
            }
        }
    }

    Ok(TextPiece::FormattedText {
        text,
        formatting: text_formatting,
    })
}

fn parse_potential_list_item(tokenizer: &mut MultipeekTokenizer) -> Option<Result<TextPiece>> {
    let offset = if !tokenizer.is_at_start() {
        if DO_PARSER_DEBUG_PRINTS {
            println!("parse_potential_list_item is not at start");
        }
        assert_eq!(tokenizer.peek(0).0, Token::Newline, "parse_potential_list_item should only be called at the beginning of the page or after a newline.");
        1
    } else {
        if DO_PARSER_DEBUG_PRINTS {
            println!("parse_potential_list_item is at start");
        }
        0
    };

    let mut list_prefix = String::new();

    // parse list_prefix
    while let token @ (Token::Colon | Token::Semicolon | Token::Star | Token::Sharp) =
        &tokenizer.peek(offset).0
    {
        list_prefix.push_str(token.to_str());
        tokenizer.next();
    }

    if list_prefix.is_empty() {
        return None;
    }

    for _ in 0..offset {
        tokenizer.next();
    }
    let text = match parse_list_item_text(tokenizer) {
        Ok(text) => text,
        Err(error) => return Some(Err(error)),
    };

    Some(Ok(TextPiece::ListItem { list_prefix, text }))
}

fn parse_list_item_text(tokenizer: &mut MultipeekTokenizer) -> Result<Text> {
    let mut text = Text::new();

    // parse text
    loop {
        let (token, text_position) = tokenizer.peek(0);
        match token {
            token @ (Token::Text(_)
            | Token::MultiEquals(_)
            | Token::Colon
            | Token::Semicolon
            | Token::Star
            | Token::Sharp) => {
                text.extend_with_text(token.to_str());
                tokenizer.next();
            }
            Token::DoubleOpenBrace => text.pieces.push(parse_double_brace_expression(tokenizer)?),
            Token::DoubleOpenBracket => text.pieces.push(parse_internal_link(tokenizer)?),
            token @ (Token::DoubleApostrophe
            | Token::TripleApostrophe
            | Token::QuintupleApostrophe) => {
                let text_formatting = token.as_text_formatting();
                text.pieces
                    .push(parse_formatted_text(tokenizer, text_formatting)?);
            }
            Token::Newline | Token::Eof => {
                tokenizer.next();
                break;
            }
            token @ (Token::DoubleCloseBrace | Token::DoubleCloseBracket | Token::VerticalBar) => {
                return Err(ParserErrorKind::UnexpectedTokenInListItem {
                    token: token.to_string(),
                }
                .into_parser_error(*text_position));
            }
        }
    }

    Ok(text)
}
