use std::fs::{self, read_to_string};

use html_parser::parse::parse_html;

#[test]
fn test() {
    let content = read_to_string("tests/data/index.html").unwrap();
    let tree = parse_html(&content).unwrap_or_else(|err| panic!("{err}"));
    let formatted = format!("{tree}");
    fs::write(".out.html", formatted).expect("Permission denied: failed to write to directory.");
    assert!(
        format!("{tree}") == content,
        "Error occurred. Output written in .out.html. Use diff to see the problem."
    );
}
