use crate::error::ParserErrorKind;
use crate::level_stack::LevelStack;
use crate::tokenizer::{MultipeekTokenizer, Token, Tokenizer};
use crate::wikitext::{Attribute, Headline, Line, Text, TextFormatting, TextPiece, Wikitext};
use crate::ParserError;
use log::debug;
use std::mem;

pub const MAX_SECTION_DEPTH: usize = 6;

#[cfg(not(test))]
static DO_PARSER_DEBUG_PRINTS: bool = false;
#[cfg(test)]
static DO_PARSER_DEBUG_PRINTS: bool = true;

/// Parse textual wikitext into a semantic representation.
pub fn parse_wikitext(
    wikitext: &str,
    headline: String,
    mut error_consumer: impl FnMut(ParserError),
) -> Wikitext {
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
            if let Some(headline) = parse_potential_headline(&mut tokenizer, &mut error_consumer) {
                level_stack.append_headline(headline);
                continue;
            }
        } else if token == &Token::Eof {
            break;
        }

        level_stack.append_line(parse_line(&mut tokenizer, &mut error_consumer));
    }

    Wikitext {
        root_section: level_stack.into_root_section(),
    }
}

fn parse_line(
    tokenizer: &mut MultipeekTokenizer,
    error_consumer: &mut impl FnMut(ParserError),
) -> Line {
    debug_assert_eq!(parse_potential_headline(tokenizer, error_consumer), None);

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
        let text = parse_text_until(
            tokenizer,
            error_consumer,
            Text::new(),
            &mut text_formatting,
            &|token: &Token<'_>| matches!(token, Token::Newline | Token::Eof),
        );
        let (_, text_position) = tokenizer.next();
        if text_formatting != TextFormatting::Normal {
            debug!("Line contains unclosed text formatting expression at {text_position:?}");
        }
        Line::List { list_prefix, text }
    } else {
        let mut text_formatting = TextFormatting::Normal;
        let text = parse_text_until(
            tokenizer,
            error_consumer,
            Text::new(),
            &mut text_formatting,
            &|token| matches!(token, Token::Newline | Token::Eof),
        );
        let (_, text_position) = tokenizer.next();
        if text_formatting != TextFormatting::Normal {
            debug!("Line contains unclosed text formatting expression at {text_position:?}");
        }
        Line::Normal { text }
    }
}

fn parse_text_until(
    tokenizer: &mut MultipeekTokenizer,
    error_consumer: &mut impl FnMut(ParserError),
    mut prefix: Text,
    text_formatting: &mut TextFormatting,
    terminator: &impl Fn(&Token<'_>) -> bool,
) -> Text {
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
            Token::DoubleOpenBrace => prefix.pieces.push(parse_double_brace_expression(
                tokenizer,
                error_consumer,
                text_formatting,
            )),
            Token::DoubleOpenBracket => {
                prefix = parse_internal_link(tokenizer, error_consumer, prefix, text_formatting);
            }
            Token::NoWikiOpen => {
                prefix = parse_nowiki(tokenizer, error_consumer, prefix, text_formatting);
            }
            Token::DoubleCloseBrace => {
                error_consumer(
                    ParserErrorKind::UnmatchedDoubleCloseBrace.into_parser_error(*text_position),
                );
                prefix.extend_with_formatted_text(*text_formatting, token.to_str());
                tokenizer.next();
            }
            Token::DoubleCloseBracket => {
                error_consumer(
                    ParserErrorKind::UnmatchedDoubleCloseBracket.into_parser_error(*text_position),
                );
                prefix.extend_with_formatted_text(*text_formatting, token.to_str());
                tokenizer.next();
            }
            Token::NoWikiClose => {
                error_consumer(
                    ParserErrorKind::UnmatchedNoWikiClose.into_parser_error(*text_position),
                );
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
                error_consumer(ParserErrorKind::UnexpectedEof.into_parser_error(*text_position));
                break;
            }
        }
    }

    prefix
}

fn parse_nowiki(
    tokenizer: &mut MultipeekTokenizer,
    error_consumer: &mut impl FnMut(ParserError),
    mut text: Text,
    text_formatting: &mut TextFormatting,
) -> Text {
    tokenizer.expect(&Token::NoWikiOpen).unwrap();

    loop {
        if DO_PARSER_DEBUG_PRINTS {
            println!("parse_nowiki token: {:?}", tokenizer.peek(0));
        }
        let (token, text_position) = tokenizer.peek(0);

        match token {
            Token::NoWikiClose => {
                tokenizer.next();
                break;
            }
            Token::Eof => {
                error_consumer(
                    ParserErrorKind::UnmatchedNoWikiOpen.into_parser_error(*text_position),
                );
                break;
            }
            token => {
                text.extend_with_formatted_text(*text_formatting, token.to_str());
                tokenizer.next();
            }
        }
    }

    text
}

fn parse_potential_headline(
    tokenizer: &mut MultipeekTokenizer,
    error_consumer: &mut impl FnMut(ParserError),
) -> Option<Headline> {
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
                error_consumer(
                    ParserErrorKind::SecondRootSection {
                        label: label.clone(),
                    }
                    .into_parser_error(text_position),
                );
            }

            Some(Headline {
                label,
                level: prefix_length.try_into().unwrap(),
            })
        } else {
            None
        }
    } else {
        None
    }
}

fn parse_double_brace_expression(
    tokenizer: &mut MultipeekTokenizer,
    error_consumer: &mut impl FnMut(ParserError),
    text_formatting: &mut TextFormatting,
) -> TextPiece {
    tokenizer.expect(&Token::DoubleOpenBrace).unwrap();
    if DO_PARSER_DEBUG_PRINTS {
        println!(
            "parse_double_brace_expression initial token: {:?}",
            tokenizer.peek(0)
        );
    }
    let tag = parse_tag(tokenizer, error_consumer);
    let mut attributes = Vec::new();

    // parse attributes
    loop {
        if DO_PARSER_DEBUG_PRINTS {
            println!(
                "parse_double_brace_expression token: {:?}",
                tokenizer.peek(0)
            );
        }
        let (token, text_position) = tokenizer.peek(0);
        match token {
            Token::VerticalBar => {
                attributes.push(parse_attribute(tokenizer, error_consumer, text_formatting))
            }
            Token::DoubleCloseBrace => {
                tokenizer.next();
                break;
            }
            token @ (Token::Text(_)
            | Token::Equals
            | Token::DoubleOpenBrace
            | Token::DoubleOpenBracket
            | Token::NoWikiOpen
            | Token::DoubleCloseBracket
            | Token::NoWikiClose
            | Token::Apostrophe
            | Token::Newline
            | Token::Colon
            | Token::Semicolon
            | Token::Star
            | Token::Sharp) => {
                error_consumer(
                    ParserErrorKind::UnexpectedToken {
                        expected: "| or }}".to_string(),
                        actual: token.to_string(),
                    }
                    .into_parser_error(*text_position),
                );
                tokenizer.next();
            }
            Token::Eof => {
                error_consumer(
                    ParserErrorKind::UnmatchedDoubleOpenBrace.into_parser_error(*text_position),
                );
                break;
            }
        }
    }

    TextPiece::DoubleBraceExpression { tag, attributes }
}

fn parse_tag(
    tokenizer: &mut MultipeekTokenizer,
    error_consumer: &mut impl FnMut(ParserError),
) -> Text {
    if DO_PARSER_DEBUG_PRINTS {
        println!("parse_tag initial token: {:?}", tokenizer.peek(0));
    }
    let text_position = tokenizer.peek(0).1;
    let mut text_formatting = TextFormatting::Normal;
    let mut tag = Text::new();

    loop {
        tag = parse_text_until(
            tokenizer,
            error_consumer,
            tag,
            &mut text_formatting,
            &|token: &Token<'_>| {
                matches!(
                    token,
                    Token::DoubleCloseBrace
                        | Token::VerticalBar
                        | Token::DoubleOpenBracket
                        | Token::Eof
                )
            },
        );
        let (token, text_position) = tokenizer.peek(0);
        match token {
            Token::DoubleCloseBrace | Token::VerticalBar => break,
            token @ Token::DoubleOpenBracket => {
                error_consumer(
                    ParserErrorKind::UnexpectedTokenInTag {
                        token: token.to_string(),
                    }
                    .into_parser_error(*text_position),
                );
                tag.extend_with_formatted_text(text_formatting, token.to_str());
                tokenizer.next();
            }
            Token::Eof => {
                error_consumer(
                    ParserErrorKind::UnmatchedDoubleOpenBrace.into_parser_error(*text_position),
                );
                break;
            }
            token => unreachable!("Not a stop token above: {token:?}"),
        }
    }

    if text_formatting != TextFormatting::Normal {
        error_consumer(
            ParserErrorKind::UnclosedTextFormatting {
                formatting: text_formatting,
            }
            .into_parser_error(text_position),
        );
    }

    tag.trim_self();
    tag
}

fn parse_attribute(
    tokenizer: &mut MultipeekTokenizer,
    error_consumer: &mut impl FnMut(ParserError),
    text_formatting: &mut TextFormatting,
) -> Attribute {
    tokenizer.expect(&Token::VerticalBar).unwrap();
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
                name.as_mut().unwrap().push_str(text);
                tokenizer.next();
            }
            Token::Newline => {
                name.as_mut().unwrap().push('\n');
                tokenizer.next();
            }
            Token::Equals => {
                tokenizer.next();
                break;
            }
            Token::DoubleOpenBrace
            | Token::DoubleOpenBracket
            | Token::NoWikiOpen
            | Token::DoubleCloseBrace
            | Token::NoWikiClose
            | Token::VerticalBar
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
                error_consumer(
                    ParserErrorKind::UnexpectedTokenInParameter {
                        token: token.to_string(),
                    }
                    .into_parser_error(*text_position),
                );
                name.as_mut().unwrap().push_str(token.to_str());
                tokenizer.next();
            }
            Token::Eof => {
                error_consumer(
                    ParserErrorKind::UnmatchedDoubleOpenBrace.into_parser_error(*text_position),
                );
                break;
            }
        }
    }

    // parse value
    let mut value = parse_text_until(
        tokenizer,
        error_consumer,
        value,
        text_formatting,
        &|token: &Token<'_>| matches!(token, Token::VerticalBar | Token::DoubleCloseBrace),
    );

    // whitespace is stripped from named attribute names and values, but not from unnamed attributes
    if let Some(name) = &mut name {
        *name = name.trim().to_string();
        value.trim_self();
    }

    Attribute { name, value }
}

fn parse_internal_link(
    tokenizer: &mut MultipeekTokenizer,
    error_consumer: &mut impl FnMut(ParserError),
    mut text: Text,
    text_formatting: &mut TextFormatting,
) -> Text {
    tokenizer.expect(&Token::DoubleOpenBracket).unwrap();
    let surrounding_depth = if tokenizer.peek(0).0 == Token::DoubleOpenBracket {
        tokenizer.next();
        1
    } else {
        0
    };
    let mut target = Text::new();
    let mut options = Vec::new();
    let mut label = None;

    // parse target
    target = parse_text_until(
        tokenizer,
        error_consumer,
        target,
        text_formatting,
        &|token: &Token<'_>| {
            matches!(
                token,
                Token::DoubleCloseBracket
                    | Token::VerticalBar
                    | Token::DoubleCloseBrace
                    | Token::DoubleOpenBracket
                    | Token::Newline
                    | Token::Eof
            )
        },
    );
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
        | Token::Equals
        | Token::DoubleOpenBrace
        | Token::NoWikiOpen
        | Token::NoWikiClose) => {
            unreachable!("Not a stop token above: {token:?}");
        }
        Token::DoubleCloseBracket => {
            tokenizer.next();
        }
        Token::VerticalBar => {
            tokenizer.next();
            label = Some(Text::new());
        }
        token @ (Token::Newline | Token::Eof) => {
            error_consumer(
                ParserErrorKind::UnmatchedDoubleOpenBracket.into_parser_error(*text_position),
            );
            if token != &Token::Eof {
                text.extend_with_formatted_text(*text_formatting, token.to_str());
            }
            tokenizer.next();
        }
        token @ (Token::DoubleCloseBrace | Token::DoubleOpenBracket) => {
            error_consumer(
                ParserErrorKind::UnexpectedTokenInLink {
                    token: token.to_string(),
                }
                .into_parser_error(*text_position),
            );
            text.extend_with_formatted_text(*text_formatting, token.to_str());
            tokenizer.next();
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
                token @ (Token::Equals | Token::Text(_)) => {
                    label.extend_with_formatted_text(*text_formatting, token.to_str());
                    tokenizer.next();
                }
                Token::VerticalBar => {
                    let mut new_label = Text::new();
                    mem::swap(&mut label, &mut new_label);
                    if new_label.pieces.is_empty() {
                        options.push(Default::default());
                    } else {
                        options.push(new_label);
                    }
                    tokenizer.next();
                }
                Token::DoubleCloseBracket => {
                    tokenizer.next();
                    link_finished = true;
                    break;
                }
                Token::Apostrophe => {
                    label = parse_text_until(
                        tokenizer,
                        error_consumer,
                        label,
                        text_formatting,
                        &|token| !matches!(token, Token::Apostrophe),
                    );
                }
                Token::DoubleOpenBrace
                | Token::DoubleOpenBracket
                | Token::NoWikiOpen
                | Token::NoWikiClose
                | Token::Colon
                | Token::Semicolon
                | Token::Star
                | Token::Sharp
                | Token::Newline => {
                    break;
                }
                token @ Token::DoubleCloseBrace => {
                    error_consumer(
                        ParserErrorKind::UnexpectedTokenInLinkLabel {
                            token: token.to_string(),
                        }
                        .into_parser_error(*text_position),
                    );
                    label.extend_with_formatted_text(*text_formatting, token.to_str());
                    tokenizer.next();
                }
                Token::Eof => {
                    error_consumer(
                        ParserErrorKind::UnmatchedDoubleOpenBracket
                            .into_parser_error(*text_position),
                    );
                    break;
                }
            }
        }

        if !link_finished {
            // parse label
            loop {
                label = parse_text_until(
                    tokenizer,
                    error_consumer,
                    label,
                    text_formatting,
                    &|token: &Token<'_>| {
                        matches!(
                            token,
                            Token::DoubleCloseBracket
                                | Token::VerticalBar
                                | Token::Newline
                                | Token::Eof
                        )
                    },
                );

                let (token, text_position) = tokenizer.peek(0);
                match token {
                    Token::DoubleCloseBracket => {
                        tokenizer.next();
                        break;
                    }
                    token @ Token::VerticalBar => {
                        error_consumer(
                            ParserErrorKind::UnexpectedTokenInLinkLabel {
                                token: token.to_string(),
                            }
                            .into_parser_error(*text_position),
                        );
                        label.extend_with_formatted_text(*text_formatting, token.to_str());
                        tokenizer.next();
                    }
                    Token::Newline | Token::Eof => {
                        error_consumer(
                            ParserErrorKind::UnmatchedDoubleOpenBracket
                                .into_parser_error(*text_position),
                        );
                        tokenizer.next();
                        break;
                    }
                    token => unreachable!("Not a stop token above: {token:?}"),
                }
            }

            label
        } else {
            label
        }
    });

    // update text
    for _ in 0..surrounding_depth {
        text.extend_with_formatted_text(*text_formatting, "[[");
    }
    text.pieces.push(TextPiece::InternalLink {
        target,
        options,
        label,
    });
    for _ in 0..surrounding_depth {
        let (token, text_position) = tokenizer.peek(0);
        match token {
            token @ Token::DoubleCloseBracket => {
                text.extend_with_formatted_text(*text_formatting, token.to_str());
                tokenizer.next();
            }
            _ => {
                error_consumer(
                    ParserErrorKind::UnmatchedDoubleOpenBracket.into_parser_error(*text_position),
                );
            }
        }
    }

    text
}
