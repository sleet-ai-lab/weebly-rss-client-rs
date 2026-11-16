//! Normalises whitespace in extracted text.

/// Normalises whitespace in extracted text.
///
/// Runs of spaces and tabs collapse into a single space, and runs of newlines
/// collapse into a single newline. Non-breaking spaces, which Weebly uses
/// liberally in its post bodies, become ordinary spaces. The result is
/// trimmed at both ends.
///
/// ```
/// # use weebly_rss_client_rs::util::collapse_whitespace;
/// assert_eq!(collapse_whitespace("  a   b \n\n c  "), "a b\nc");
/// assert_eq!(collapse_whitespace("a\u{a0}b"), "a b");
/// ```
pub fn collapse_whitespace(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut pending_space = false;
    let mut newline_run = 0usize;

    for ch in text.chars() {
        match ch {
            '\n' | '\r' => {
                pending_space = false;
                newline_run += 1;
            }
            // A non-breaking space is a space for every purpose here.
            '\u{a0}' | '\u{202f}' => {
                if newline_run == 0 {
                    pending_space = true;
                }
            }
            ' ' | '\t' => {
                if newline_run == 0 {
                    pending_space = true;
                }
            }
            _ => {
                if newline_run > 0 {
                    if !out.is_empty() {
                        out.push('\n');
                    }
                } else if pending_space && !out.is_empty() && !out.ends_with('\n') {
                    out.push(' ');
                }
                newline_run = 0;
                pending_space = false;
                out.push(ch);
            }
        }
    }

    out.lines()
        .map(|line| line.trim_end())
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}
