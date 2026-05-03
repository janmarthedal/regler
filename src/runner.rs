/// Strip a line comment (`#` and everything after) while respecting double-quoted strings.
pub fn strip_comment(line: &str) -> &str {
    let mut in_string = false;
    for (i, c) in line.char_indices() {
        match c {
            '"' => in_string = !in_string,
            '#' if !in_string => return &line[..i],
            _ => {}
        }
    }
    line
}

/// Group raw file lines into complete statements using indentation-based continuation.
///
/// A non-empty line whose indent (leading whitespace character count) strictly
/// exceeds the first line's indent belongs to the same statement. Blank lines
/// (including comment-only lines) are inert — they do not end a statement.
pub fn collect_statements(lines: Vec<String>) -> Vec<String> {
    let mut stmts = Vec::new();
    let mut current: Option<(usize, String)> = None; // (first_line_indent, accumulated text)

    for line in lines {
        let effective = strip_comment(&line).trim_end();
        let content = effective.trim_start();
        if content.is_empty() {
            continue; // blank and comment-only lines are inert
        }
        let indent: usize = effective.chars().take_while(|c| c.is_whitespace()).count();

        match &mut current {
            None => {
                current = Some((indent, content.to_string()));
            }
            Some((first_indent, stmt)) => {
                if indent > *first_indent {
                    stmt.push(' ');
                    stmt.push_str(content);
                } else {
                    stmts.push(current.take().unwrap().1);
                    current = Some((indent, content.to_string()));
                }
            }
        }
    }
    if let Some((_, stmt)) = current {
        stmts.push(stmt);
    }
    stmts
}
