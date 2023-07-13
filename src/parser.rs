use crate::error::{ParserErrorKind, Result};
use crate::level_stack::LevelStack;
use crate::tokenizer::{MultipeekTokenizer, Token, Tokenizer};
use crate::wikitext::{Attribute, Headline, Line, Text, TextFormatting, TextPiece, Wikitext};
use log::debug;
use std::mem;

pub const MAX_SECTION_DEPTH: usize = 6;

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

        if matches!(token, Token::Equals) {
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
    debug_assert_eq!(
        parse_potential_headline(tokenizer)
            .map(|result| result.map_err(|error| format!("{error:?}"))),
        None
    );

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
        let mut text_formatting = TextFormatting::Normal;
        let text = parse_text_until(tokenizer, Text::new(), &mut text_formatting, |token| {
            matches!(token, Token::Newline | Token::Eof)
        })?;
        let (_, text_position) = tokenizer.next();
        if text_formatting != TextFormatting::Normal {
            debug!("Line contains unclosed text formatting expression at {text_position:?}");
        }
        Ok(Line::List { list_prefix, text })
    } else {
        let mut text_formatting = TextFormatting::Normal;
        let text = parse_text_until(tokenizer, Text::new(), &mut text_formatting, |token| {
            matches!(token, Token::Newline | Token::Eof)
        })?;
        let (_, text_position) = tokenizer.next();
        if text_formatting != TextFormatting::Normal {
            debug!("Line contains unclosed text formatting expression at {text_position:?}");
        }
        Ok(Line::Normal { text })
    }
}

fn parse_text_until(
    tokenizer: &mut MultipeekTokenizer,
    mut prefix: Text,
    text_formatting: &mut TextFormatting,
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
            | Token::Equals
            | Token::Colon
            | Token::Semicolon
            | Token::Star
            | Token::Sharp
            | Token::Newline
            | Token::VerticalBar) => {
                prefix.extend_with_formatted_text(*text_formatting, token.to_str());
                tokenizer.next();
            }
            Token::DoubleOpenBrace => prefix
                .pieces
                .push(parse_double_brace_expression(tokenizer, text_formatting)?),
            Token::DoubleOpenBracket => prefix
                .pieces
                .push(parse_internal_link(tokenizer, text_formatting)?),
            Token::DoubleCloseBrace => {
                debug!("Line contains unmatched double close brace at {text_position:?}");
                prefix.extend_with_formatted_text(*text_formatting, token.to_str());
                tokenizer.next();
            }
            Token::DoubleCloseBracket => {
                debug!("Line contains unmatched double close bracket at {text_position:?}");
                prefix.extend_with_formatted_text(*text_formatting, token.to_str());
                tokenizer.next();
            }
            Token::Apostrophe => {
                tokenizer.peek(4);
                let apostrophe_prefix_length = (0..5)
                    .take_while(|i| tokenizer.peek(*i).0 == Token::Apostrophe)
                    .count();
                if apostrophe_prefix_length == 1 {
                    prefix.extend_with_formatted_text(*text_formatting, "'");
                    tokenizer.next();
                } else {
                    let apostrophe_prefix_length = if apostrophe_prefix_length == 4 {
                        3
                    } else {
                        apostrophe_prefix_length
                    };
                    *text_formatting = text_formatting.next_formatting(apostrophe_prefix_length);
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
    if DO_PARSER_DEBUG_PRINTS {
        tokenizer.peek(2 * MAX_SECTION_DEPTH + 2);
        println!(
            "parse_potential_headline initial tokens: {:?}",
            (0..2 * MAX_SECTION_DEPTH + 3)
                .map(|i| tokenizer.repeek(i))
                .collect::<Vec<_>>()
        );
    }

    let text_position = tokenizer.peek(0).1;
    let prefix_length = (0..MAX_SECTION_DEPTH)
        .take_while(|i| tokenizer.peek(*i).0 == Token::Equals)
        .count();
    if prefix_length == 0 {
        return None;
    }

    tokenizer.peek(prefix_length * 2 + 2);
    let Token::Text(text) = &tokenizer.repeek(prefix_length).unwrap().0 else {
        return None;
    };
    let suffix_length = ((prefix_length + 1)..=(2 * prefix_length + 1))
        .take_while(|i| tokenizer.repeek(*i).unwrap().0 == Token::Equals)
        .count();

    if prefix_length == suffix_length {
        let whitespace_after_headline = match &tokenizer.repeek(prefix_length * 2 + 1).unwrap().0 {
            Token::Text(text) => {
                debug_assert!(text.chars().all(|c| c != '\n'));
                if text.chars().all(|c| c.is_ascii_whitespace()) {
                    if matches!(
                        tokenizer.repeek(prefix_length * 2 + 2).unwrap().0,
                        Token::Newline | Token::Eof
                    ) {
                        Some(2)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            Token::Newline | Token::Eof => Some(1),
            _ => None,
        };

        if let Some(whitespace_after_headline) = whitespace_after_headline {
            let label = text.trim().to_string();
            for _ in 0..2 * prefix_length + 1 + whitespace_after_headline {
                tokenizer.next();
            }

            if prefix_length == 1 {
                debug!("Line contains second root section at {text_position:?}");
            }

            Some(Ok(Headline {
                label,
                level: prefix_length.try_into().unwrap(),
            }))
        } else {
            None
        }
    } else {
        None
    }
}

fn parse_double_brace_expression(
    tokenizer: &mut MultipeekTokenizer,
    text_formatting: &mut TextFormatting,
) -> Result<TextPiece> {
    tokenizer.expect(&Token::DoubleOpenBrace)?;
    if DO_PARSER_DEBUG_PRINTS {
        println!(
            "parse_double_brace_expression initial token: {:?}",
            tokenizer.peek(0)
        );
    }
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
            | Token::Equals
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

fn parse_tag(tokenizer: &mut MultipeekTokenizer) -> Result<Text> {
    if DO_PARSER_DEBUG_PRINTS {
        println!("parse_tag initial token: {:?}", tokenizer.peek(0));
    }
    let text_position = tokenizer.peek(0).1;
    let mut text_formatting = TextFormatting::Normal;
    let mut tag = parse_text_until(tokenizer, Text::new(), &mut text_formatting, |token| {
        matches!(
            token,
            Token::DoubleCloseBrace | Token::VerticalBar | Token::DoubleOpenBracket
        )
    })
    .map_err(|error| error.annotate_self("parse_tag".to_string()))?;

    if text_formatting != TextFormatting::Normal {
        return Err(ParserErrorKind::UnclosedTextFormatting {
            formatting: text_formatting,
        }
        .into_parser_error(text_position));
    }
    let (token, text_position) = tokenizer.peek(0);
    match token {
        Token::DoubleCloseBrace | Token::VerticalBar => {}
        token @ Token::DoubleOpenBracket => {
            return Err(ParserErrorKind::UnexpectedTokenInTag {
                token: token.to_string(),
            }
            .into_parser_error(*text_position))
        }
        token => unreachable!("Not a stop token above: {token:?}"),
    }

    tag.trim_self();
    Ok(tag)
}

fn parse_attribute(
    tokenizer: &mut MultipeekTokenizer,
    text_formatting: &mut TextFormatting,
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
            Token::Equals => {
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
                    formatting: *text_formatting,
                });
                break;
            }
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
    text_formatting: &mut TextFormatting,
) -> Result<TextPiece> {
    tokenizer.expect(&Token::DoubleOpenBracket)?;
    let mut target = Text::new();
    let mut options = Vec::new();
    let mut label = None;

    // parse target
    target = parse_text_until(tokenizer, target, text_formatting, |token| {
        matches!(
            token,
            Token::DoubleCloseBracket
                | Token::VerticalBar
                | Token::DoubleOpenBrace
                | Token::DoubleCloseBrace
                | Token::DoubleOpenBracket
                | Token::Newline
        )
    })
    .map_err(|error| error.annotate_self("parse_internal_link target".to_string()))?;
    if DO_PARSER_DEBUG_PRINTS {
        println!("parse_link target token: {:?}", tokenizer.peek(0));
    }
    let (token, text_position) = tokenizer.peek(0);
    match token {
        token @ (Token::Text(_)
        | Token::Colon
        | Token::Sharp
        | Token::Semicolon
        | Token::Star
        | Token::Apostrophe
        | Token::Eof
        | Token::Equals) => {
            unreachable!("Not a stop token above: {token:?}");
        }
        Token::DoubleCloseBracket => {
            tokenizer.next();
        }
        Token::VerticalBar => {
            tokenizer.next();
            label = Some(Text::new());
        }
        Token::Newline => {
            debug!("Line contains unclosed internal link at {text_position:?}");
            tokenizer.next();
        }
        token @ (Token::DoubleOpenBrace | Token::DoubleCloseBrace | Token::DoubleOpenBracket) => {
            return Err(ParserErrorKind::UnexpectedTokenInLink {
                token: token.to_string(),
            }
            .into_parser_error(*text_position))
        }
    }

    // parse options and label
    let label = label
        .map(|mut label| {
            let mut link_finished = false;

            // parse options
            loop {
                if DO_PARSER_DEBUG_PRINTS {
                    println!("parse_link options token: {:?}", tokenizer.peek(0));
                }
                let (token, text_position) = tokenizer.peek(0);
                match token {
                    Token::Text(text) => {
                        label.extend_with_formatted_text(*text_formatting, text);
                        tokenizer.next();
                    }
                    Token::VerticalBar => {
                        let mut new_label = Text::new();
                        mem::swap(&mut label, &mut new_label);
                        if new_label.pieces.is_empty() {
                            options.push(Default::default());
                        } else {
                            assert_eq!(new_label.pieces.len(), 1);
                            let TextPiece::Text { text, .. } =
                                new_label.pieces.into_iter().next().unwrap()
                            else {
                                unreachable!("Only text is ever inserted into link options");
                            };
                            options.push(text);
                        }
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
                    | Token::Sharp
                    | Token::Newline
                    | Token::Equals => {
                        break;
                    }
                    token @ Token::DoubleCloseBrace => {
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
                let label = parse_text_until(tokenizer, label, text_formatting, |token| {
                    matches!(token, Token::DoubleCloseBracket)
                })
                .map_err(|error| error.annotate_self("parse_internal_link label".to_string()))?;
                let next_token = tokenizer.expect(&Token::DoubleCloseBracket);
                debug_assert!(next_token.is_ok());
                label
            } else {
                label
            })
        })
        .transpose()?;

    Ok(TextPiece::InternalLink {
        target,
        options,
        label,
    })
}
