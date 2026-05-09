use std::fs::read_to_string;

use html_filter::*;

use super::test_maker;

#[test]
fn parse() {
    let content = read_to_string("tests/data/index.html").expect("Missing tests/data/index.html");
    let tree = Html::parse(&content).unwrap_or_else(|err| panic!("{err}"));
    test_maker("parse", &content, &tree, "", true);
}

#[test]
fn no_filter() {
    let content = read_to_string("tests/data/index.html").expect("Missing tests/data/index.html");
    let tree = Html::parse(&content).unwrap_or_else(|err| panic!("{err}")).filter(&Filter::new());
    test_maker("no_filter", &content, &tree, "", true);
}
