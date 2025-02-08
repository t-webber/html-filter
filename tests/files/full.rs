use std::fs::read_to_string;

use html_parser::parse::parse_html;

use super::test_maker;

#[test]
fn index() {
    let content = read_to_string("tests/data/index.html").unwrap();
    let tree = parse_html(&content).unwrap_or_else(|err| panic!("{err}"));
    test_maker("full", &content, tree);
}
