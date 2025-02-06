use std::fs::{self, read_to_string};

use html_parser::parse::parse_html;

fn format_html(html: &str) -> String {
    let mut old = html.replace('/', " /");
    loop {
        let out = old.replace("  ", " ");
        if out == old {
            break;
        }
        old = out;
    }
    old
}

#[test]
fn test() {
    let content = read_to_string("tests/data/index.html").unwrap();
    let tree = parse_html(&content).unwrap_or_else(|err| panic!("{err}"));
    let formatted_input = format_html(&content);
    let formatted_output = format_html(&format!("{tree}"));
    fs::write(".out.html", &formatted_output)
        .expect("Permission denied: failed to write to directory.");
    assert!(
        formatted_input == formatted_output,
        "Error occurred. Output written in .out.html. Use diff to see the problem."
    );
}
