/// Test expected parsing errors.
pub mod errors;
/// Test filters on index.html.
pub mod filter;
/// Test finders on index.html.
pub mod find;
/// Test no filter keeps html intact.
pub mod full;
/// Test that ana html is parsed correctly.
pub mod matches;
/// Test filters on a smaller string.
pub mod strings;
/// Test the trimming mechanism.
pub mod trim;

use core::fmt::Debug;
use std::fs;

use html_filter::*;

fn handle_auto_closing(html: &str) -> String {
    let mut output = String::with_capacity(html.len());
    let mut tag_name = String::new();
    let mut reading = false;
    let mut last_slash = false;
    for ch in html.chars() {
        if ch == '>' && last_slash {
            output.push_str("> </");
            output.push_str(&tag_name);
            output.push('>');
            tag_name.clear();
            last_slash = false;
            reading = false;
        } else if ch == '/' {
            if last_slash {
                output.push('/');
            } else {
                last_slash = true;
            }
            reading = false;
        } else {
            if last_slash {
                last_slash = false;
                output.push('/');
            }
            if ch == '<' {
                reading = true;
                tag_name.clear();
                output.push('<');
            } else {
                if ch.is_whitespace() || ch == '!' || ch == '>' {
                    reading = false;
                }
                output.push(ch);
                if reading {
                    tag_name.push(ch);
                }
            }
        }
    }
    output
}

fn format_html(html: &str) -> String {
    let mut formatted = html
        .replace('/', " /")
        .replace('\n', " ")
        .replace("< ", "<")
        .replace(" >", ">")
        .replace('<', " <")
        .replace('>', "> ");
    loop {
        let out = formatted.replace("  ", " ");
        if out == formatted {
            break;
        }
        formatted = out;
    }
    handle_auto_closing(&formatted)
        .replace(" >", ">")
        .replace("> </meta>", ">")
        .replace("> </br>", ">")
}

fn test_maker<T: Debug>(name: &str, expected: &str, output: &Html, msg: T, simplify: bool) {
    let (formatted_input, formatted_output) = if simplify {
        (format_html(expected), format_html(&output.to_string()))
    } else {
        (expected.to_owned(), output.to_string())
    };
    if formatted_output != formatted_input {
        let output_path = format!("output.{name}.html");
        let expected_path = format!("expected.{name}.html");
        fs::write(&output_path, formatted_output.replace(' ', "\n"))
            .expect("Permission denied: failed to write to directory.");
        fs::write(&expected_path, formatted_input.replace(' ', "\n"))
            .expect("Permission denied: failed to write to directory.");
        let sep = format!("\n\x1b[33m{}\x1b[0m\n", "-".repeat(50));
        let nl = "\n";
        let output = "\x1b[33mOutput:\x1b[0m";
        let expected = "\x1b[33mExpected:\x1b[0m";
        panic!(
            "{msg:?}\n{output}{sep}{formatted_output}{sep}{nl}{expected}{sep}{formatted_input}{sep}Use `diff {output_path} {expected_path}` to \
             see the problem."
        );
    }
}
