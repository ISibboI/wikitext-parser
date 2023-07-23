use crate::{parse_wikitext, ParserErrorKind, TextPosition};

#[test]
fn test_wiktionary_pöytä() {
    let input = include_str!("pages/pouta.txt");
    let mut errors = Vec::new();
    let _parsed = parse_wikitext(
        input,
        "pöytä".to_string(),
        &mut Box::new(|error| errors.push(error)),
    );
    assert!(errors.is_empty());
    /*parsed.print_headlines();
    *for double_brace_expression in parsed.list_double_brace_expressions() {
            println!("{}", double_brace_expression);
        }
        for plain_text in parsed.list_plain_text() {
            println!("{}", plain_text);
        }*/
}

#[test]
fn test_wiktionary_free() {
    let input = include_str!("pages/free.txt");
    let mut errors = Vec::new();
    let _parsed = parse_wikitext(
        input,
        "free".to_string(),
        &mut Box::new(|error| errors.push(error)),
    );
    assert!(errors.is_empty());
    /*parsed.print_headlines();
    *for double_brace_expression in parsed.list_double_brace_expressions() {
            println!("{}", double_brace_expression);
        }
        for plain_text in parsed.list_plain_text() {
            println!("{}", plain_text);
        }*/
}

#[test]
fn test_wiktionary_cat() {
    let input = include_str!("pages/cat.txt");
    let mut errors = Vec::new();
    let _parsed = parse_wikitext(
        input,
        "cat".to_string(),
        &mut Box::new(|error| errors.push(error)),
    );
    assert!(errors.is_empty());
    /*parsed.print_headlines();
    *for double_brace_expression in parsed.list_double_brace_expressions() {
            println!("{}", double_brace_expression);
        }
        for plain_text in parsed.list_plain_text() {
            println!("{}", plain_text);
        }*/
}

#[test]
fn test_wiktionary_a() {
    let input = include_str!("pages/a.txt");
    let mut errors = Vec::new();
    let _parsed = parse_wikitext(
        input,
        "a".to_string(),
        &mut Box::new(|error| errors.push(error)),
    );
    assert!(errors.is_empty());
    /*parsed.print_headlines();
    *for double_brace_expression in parsed.list_double_brace_expressions() {
            println!("{}", double_brace_expression);
        }
        for plain_text in parsed.list_plain_text() {
            println!("{}", plain_text);
        }*/
}

#[test]
fn test_wiktionary_help_how_to_edit_a_page() {
    let input = include_str!("pages/help_how_to_edit_a_page.txt");
    let mut errors = Vec::new();
    let _parsed = parse_wikitext(
        input,
        "Help:How to edit a page".to_string(),
        &mut Box::new(|error| errors.push(error)),
    );
    assert!(errors.is_empty());
    /*parsed.print_headlines();
    *for double_brace_expression in parsed.list_double_brace_expressions() {
            println!("{}", double_brace_expression);
        }
        for plain_text in parsed.list_plain_text() {
            println!("{}", plain_text);
        }*/
}

#[test]
fn test_wiktionary_ある() {
    let input = include_str!("pages/japanese_exist.txt");
    let mut errors = Vec::new();
    let _parsed = parse_wikitext(
        input,
        "ある".to_string(),
        &mut Box::new(|error| errors.push(error)),
    );
    assert!(errors.is_empty());
    /*parsed.print_headlines();
    *for double_brace_expression in parsed.list_double_brace_expressions() {
            println!("{}", double_brace_expression);
        }
        for plain_text in parsed.list_plain_text() {
            println!("{}", plain_text);
        }*/
}

#[test]
fn test_wiktionary_arunachal_pradesh() {
    let input = include_str!("pages/arunachal_pradesh.txt");
    let mut errors = Vec::new();
    let _parsed = parse_wikitext(
        input,
        "Arunachal Pradesh".to_string(),
        &mut Box::new(|error| errors.push(error)),
    );
    assert!(errors.is_empty());
    /*parsed.print_headlines();
    *for double_brace_expression in parsed.list_double_brace_expressions() {
            println!("{}", double_brace_expression);
        }
        for plain_text in parsed.list_plain_text() {
            println!("{}", plain_text);
        }*/
}

#[test]
fn test_wiktionary_wiktionary_license_discussion() {
    let input = include_str!("pages/wiktionary_license_discussion.txt");
    let mut errors = Vec::new();
    let _parsed = parse_wikitext(
        input,
        "Wiktionary:License discussion".to_string(),
        &mut Box::new(|error| errors.push(error)),
    );
    assert!(errors.is_empty());
    /*parsed.print_headlines();
    *for double_brace_expression in parsed.list_double_brace_expressions() {
            println!("{}", double_brace_expression);
        }
        for plain_text in parsed.list_plain_text() {
            println!("{}", plain_text);
        }*/
}

#[test]
fn test_wiktionary_bone() {
    let input = include_str!("pages/bone.txt");
    let mut errors = Vec::new();
    let _parsed = parse_wikitext(
        input,
        "bone".to_string(),
        &mut Box::new(|error| errors.push(error)),
    );
    assert!(errors.is_empty());
    /*parsed.print_headlines();
    *for double_brace_expression in parsed.list_double_brace_expressions() {
            println!("{}", double_brace_expression);
        }
        for plain_text in parsed.list_plain_text() {
            println!("{}", plain_text);
        }*/
}

#[test]
fn test_wiktionary_y() {
    let input = include_str!("pages/y.txt");
    let mut errors = Vec::new();
    let _parsed = parse_wikitext(
        input,
        "y".to_string(),
        &mut Box::new(|error| errors.push(error)),
    );
    assert_eq!(
        errors,
        vec![
            ParserErrorKind::UnmatchedDoubleCloseBracket.into_parser_error(TextPosition {
                line: 942,
                column: 58
            })
        ]
    );
    /*parsed.print_headlines();
    *for double_brace_expression in parsed.list_double_brace_expressions() {
            println!("{}", double_brace_expression);
        }
        for plain_text in parsed.list_plain_text() {
            println!("{}", plain_text);
        }*/
}

#[test]
fn test_wiktionary_wiktionary_tea_room() {
    let input = include_str!("pages/wiktionary_tea_room.txt");
    let mut errors = Vec::new();
    let _parsed = parse_wikitext(
        input,
        "Wiktionary:Tea room".to_string(),
        &mut Box::new(|error| errors.push(error)),
    );
    assert!(errors.is_empty());
    /*parsed.print_headlines();
    *for double_brace_expression in parsed.list_double_brace_expressions() {
            println!("{}", double_brace_expression);
        }
        for plain_text in parsed.list_plain_text() {
            println!("{}", plain_text);
        }*/
}

#[test]
fn test_wiktionary_abyssinian() {
    let input = include_str!("pages/abyssinian.txt");
    let mut errors = Vec::new();
    let _parsed = parse_wikitext(
        input,
        "Abyssinian".to_string(),
        &mut Box::new(|error| errors.push(error)),
    );
    assert_eq!(
        errors,
        vec![
            ParserErrorKind::UnmatchedDoubleOpenBracket
                .into_parser_error(TextPosition::new(20, 70)),
            ParserErrorKind::UnmatchedDoubleCloseBracket
                .into_parser_error(TextPosition::new(21, 1)),
            ParserErrorKind::UnmatchedDoubleOpenBracket
                .into_parser_error(TextPosition::new(69, 70)),
            ParserErrorKind::UnmatchedDoubleCloseBracket
                .into_parser_error(TextPosition::new(70, 1)),
        ]
    );
    /*parsed.print_headlines();
    *for double_brace_expression in parsed.list_double_brace_expressions() {
            println!("{}", double_brace_expression);
        }
        for plain_text in parsed.list_plain_text() {
            println!("{}", plain_text);
        }*/
}

#[test]
fn test_wiktionary_o() {
    let input = include_str!("pages/o.txt");
    let mut errors = Vec::new();
    let _parsed = parse_wikitext(
        input,
        "o".to_string(),
        &mut Box::new(|error| errors.push(error)),
    );
    assert_eq!(errors, vec![]);
    /*parsed.print_headlines();
    *for double_brace_expression in parsed.list_double_brace_expressions() {
            println!("{}", double_brace_expression);
        }
        for plain_text in parsed.list_plain_text() {
            println!("{}", plain_text);
        }*/
}

#[test]
fn test_wiktionary_finger() {
    let input = include_str!("pages/finger.txt");
    let mut errors = Vec::new();
    let _parsed = parse_wikitext(
        input,
        "finger".to_string(),
        &mut Box::new(|error| errors.push(error)),
    );
    assert!(errors.is_empty());
    /*parsed.print_headlines();
    *for double_brace_expression in parsed.list_double_brace_expressions() {
            println!("{}", double_brace_expression);
        }
        for plain_text in parsed.list_plain_text() {
            println!("{}", plain_text);
        }*/
}

#[test]
fn test_wiktionary_appendix_indo_iranian_swadesh_lists() {
    let input = include_str!("pages/appendix_indo_iranian_swadesh_lists.txt");
    let mut errors = Vec::new();
    let _parsed = parse_wikitext(
        input,
        "Appendix:Indo-Iranian Swadesh lists".to_string(),
        &mut Box::new(|error| errors.push(error)),
    );
    assert_eq!(
        errors,
        vec![
            ParserErrorKind::UnmatchedDoubleOpenBracket.into_parser_error(TextPosition {
                line: 2715,
                column: 40
            })
        ]
    );
    /*parsed.print_headlines();
    *for double_brace_expression in parsed.list_double_brace_expressions() {
            println!("{}", double_brace_expression);
        }
        for plain_text in parsed.list_plain_text() {
            println!("{}", plain_text);
        }*/
}

#[test]
fn test_wiktionary_wiktionary_namespace() {
    let input = include_str!("pages/wiktionary_namespace.txt");
    let mut errors = Vec::new();
    let _parsed = parse_wikitext(
        input,
        "Wiktionary:Namespace".to_string(),
        &mut Box::new(|error| errors.push(error)),
    );
    assert!(errors.is_empty());
    /*parsed.print_headlines();
    *for double_brace_expression in parsed.list_double_brace_expressions() {
            println!("{}", double_brace_expression);
        }
        for plain_text in parsed.list_plain_text() {
            println!("{}", plain_text);
        }*/
}

#[test]
fn test_wiktionary_media_wiki_blockedtext() {
    let input = include_str!("pages/media_wiki_blockedtext.txt");
    let mut errors = Vec::new();
    let _parsed = parse_wikitext(
        input,
        "MediaWiki:Blockedtext".to_string(),
        &mut Box::new(|error| errors.push(error)),
    );
    assert!(errors.is_empty());
    /*parsed.print_headlines();
    *for double_brace_expression in parsed.list_double_brace_expressions() {
            println!("{}", double_brace_expression);
        }
        for plain_text in parsed.list_plain_text() {
            println!("{}", plain_text);
        }*/
}

#[test]
fn test_wiktionary_lady() {
    let input = include_str!("pages/lady.txt");
    let mut errors = Vec::new();
    let _parsed = parse_wikitext(
        input,
        "lady".to_string(),
        &mut Box::new(|error| errors.push(error)),
    );
    assert_eq!(
        errors,
        vec![ParserErrorKind::UnexpectedTokenInParameter {
            token: "]]".to_string()
        }
        .into_parser_error(TextPosition {
            line: 296,
            column: 78
        })]
    );
    /*parsed.print_headlines();
    *for double_brace_expression in parsed.list_double_brace_expressions() {
            println!("{}", double_brace_expression);
        }
        for plain_text in parsed.list_plain_text() {
            println!("{}", plain_text);
        }*/
}

#[test]
fn test_wiktionary_appendix_slovak_declension_pattern_dlan() {
    let input = include_str!("pages/appendix_slovak_declension_pattern_dlan.txt");
    let mut errors = Vec::new();
    let _parsed = parse_wikitext(
        input,
        "Appendix:Slovak declension pattern dlaň".to_string(),
        &mut Box::new(|error| errors.push(error)),
    );
    assert_eq!(
        errors,
        vec![
            ParserErrorKind::UnmatchedDoubleOpenBracket.into_parser_error(TextPosition {
                line: 18,
                column: 794
            }),
            ParserErrorKind::UnmatchedDoubleOpenBracket.into_parser_error(TextPosition {
                line: 20,
                column: 19
            })
        ]
    );
    /*parsed.print_headlines();
    *for double_brace_expression in parsed.list_double_brace_expressions() {
            println!("{}", double_brace_expression);
        }
        for plain_text in parsed.list_plain_text() {
            println!("{}", plain_text);
        }*/
}

#[test]
fn test_wiktionary_rhymes_english_ik() {
    let input = include_str!("pages/rhymes_english_ik.txt");
    let mut errors = Vec::new();
    let _parsed = parse_wikitext(
        input,
        "Rhymes:English/ɪk".to_string(),
        &mut Box::new(|error| errors.push(error)),
    );
    assert_eq!(
        errors,
        vec![
            ParserErrorKind::UnmatchedDoubleOpenBracket.into_parser_error(TextPosition {
                line: 656,
                column: 63
            }),
        ]
    );
    /*parsed.print_headlines();
    *for double_brace_expression in parsed.list_double_brace_expressions() {
            println!("{}", double_brace_expression);
        }
        for plain_text in parsed.list_plain_text() {
            println!("{}", plain_text);
        }*/
}

#[test]
fn test_wiktionary_rhymes_english_aesi() {
    let input = include_str!("pages/rhymes_english_aesi.txt");
    let mut errors = Vec::new();
    let _parsed = parse_wikitext(
        input,
        "Rhymes:English/æsi".to_string(),
        &mut Box::new(|error| errors.push(error)),
    );
    assert_eq!(
        errors,
        vec![
            ParserErrorKind::UnexpectedTokenInLinkLabel {
                token: "}}".to_string()
            }
            .into_parser_error(TextPosition {
                line: 11,
                column: 90
            }),
            ParserErrorKind::UnmatchedDoubleOpenBracket.into_parser_error(TextPosition {
                line: 11,
                column: 93
            }),
        ]
    );
    /*parsed.print_headlines();
    *for double_brace_expression in parsed.list_double_brace_expressions() {
            println!("{}", double_brace_expression);
        }
        for plain_text in parsed.list_plain_text() {
            println!("{}", plain_text);
        }*/
}

#[test]
fn test_wiktionary_wiktionary_todo_template_language_code_doesnt_match_header() {
    let input =
        include_str!("pages/wiktionary_todo_template_language_code_doesnt_match_header.txt");
    let mut errors = Vec::new();
    let _parsed = parse_wikitext(
        input,
        "Wiktionary:Todo/Template language code doesn't match header".to_string(),
        |error| errors.push(error),
    );
    assert_eq!(errors, vec![]);
    /*parsed.print_headlines();
    *for double_brace_expression in parsed.list_double_brace_expressions() {
            println!("{}", double_brace_expression);
        }
        for plain_text in parsed.list_plain_text() {
            println!("{}", plain_text);
        }*/
}

#[test]
fn test_wiktionary_ab() {
    let input = include_str!("pages/ab.txt");
    let mut errors = Vec::new();
    let parsed = parse_wikitext(input, "ab".to_string(), |error| errors.push(error));
    assert_eq!(errors, vec![]);
    for language_subsection in &parsed.root_section.subsections {
        if language_subsection.headline.label == "Irish" {
            assert_eq!(language_subsection.subsections.len(), 4);
        }
    }

    //parsed.print_headlines();
    /*for double_brace_expression in parsed.list_double_brace_expressions() {
        println!("{}", double_brace_expression);
    }
    for plain_text in parsed.list_plain_text() {
        println!("{}", plain_text);
    }*/
}
