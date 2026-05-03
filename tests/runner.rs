use regler::runner::{collect_statements, strip_comment};

// --- strip_comment ---

#[test]
fn strip_comment_no_comment() {
    assert_eq!(strip_comment("fact a + b = b + a"), "fact a + b = b + a");
}

#[test]
fn strip_comment_whole_line() {
    assert_eq!(strip_comment("# this is a comment"), "");
}

#[test]
fn strip_comment_inline() {
    assert_eq!(strip_comment("let x = 1 # assign x"), "let x = 1 ");
}

#[test]
fn strip_comment_hash_inside_string() {
    // # inside a string literal must not be treated as a comment
    assert_eq!(strip_comment("import \"path#to/file.rgl\""), "import \"path#to/file.rgl\"");
}

#[test]
fn strip_comment_empty_line() {
    assert_eq!(strip_comment(""), "");
}

// --- collect_statements ---

fn stmts(lines: &[&str]) -> Vec<String> {
    collect_statements(lines.iter().map(|s| s.to_string()).collect())
}

#[test]
fn single_line_statements() {
    assert_eq!(stmts(&["let x = 1", "let y = 2"]), vec!["let x = 1", "let y = 2"]);
}

#[test]
fn blank_lines_are_inert() {
    assert_eq!(stmts(&["let x = 1", "", "let y = 2"]), vec!["let x = 1", "let y = 2"]);
}

#[test]
fn comment_only_lines_are_inert() {
    assert_eq!(
        stmts(&["# header", "let x = 1", "# between", "let y = 2"]),
        vec!["let x = 1", "let y = 2"]
    );
}

#[test]
fn continuation_joins_with_space() {
    assert_eq!(
        stmts(&[
            "fact ∀ a, b ∈ ℂ.",
            "    a + b = b + a",
        ]),
        vec!["fact ∀ a, b ∈ ℂ. a + b = b + a"]
    );
}

#[test]
fn blank_line_inside_continuation_is_inert() {
    // A blank line in the middle of a multi-line statement doesn't end it.
    assert_eq!(
        stmts(&[
            "fact ∀ a, b ∈ ℂ.",
            "",
            "    a + b = b + a",
        ]),
        vec!["fact ∀ a, b ∈ ℂ. a + b = b + a"]
    );
}

#[test]
fn two_multiline_statements() {
    let result = stmts(&[
        "fact ∀ a, b ∈ ℂ.",
        "    a + b = b + a",
        "fact ∀ x ∈ ℂ.",
        "    x + 0 = x",
    ]);
    assert_eq!(result, vec![
        "fact ∀ a, b ∈ ℂ. a + b = b + a",
        "fact ∀ x ∈ ℂ. x + 0 = x",
    ]);
}

#[test]
fn inline_comment_stripped_before_joining() {
    // The `#` comment on the first line must not swallow the continuation.
    assert_eq!(
        stmts(&[
            "fact ∀ a, b ∈ ℂ.   # some comment",
            "    a + b = b + a",
        ]),
        vec!["fact ∀ a, b ∈ ℂ. a + b = b + a"]
    );
}

#[test]
fn empty_input_yields_no_statements() {
    assert_eq!(stmts(&[]), Vec::<String>::new());
}

#[test]
fn only_comments_yield_no_statements() {
    assert_eq!(stmts(&["# a", "# b"]), Vec::<String>::new());
}

#[test]
fn single_statement_at_eof_without_trailing_blank() {
    assert_eq!(stmts(&["simplify x + 0"]), vec!["simplify x + 0"]);
}
