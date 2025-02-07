use std::fs::{self, read_to_string};

use html_parser::filter::{Filter, filter_html};
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

macro_rules! test_filter {
    ($($name:ident: $filter:expr => $expect:expr)*) => {
        $(
            #[test]
            fn $name() {
                let content = read_to_string("tests/data/index.html").unwrap();
                let mut tree = parse_html(&content).unwrap_or_else(|err| panic!("{err}"));
                filter_html(&mut tree, &$filter);
                dbg!(&tree);
                let formatted_input = format_html(&$expect);
                let formatted_output = format_html(&format!("{tree}"));
                if formatted_output != formatted_input {
                    fs::write("output.html", &formatted_output)
                        .expect("Permission denied: failed to write to directory.");
                    fs::write("expected.html", &formatted_input)
                        .expect("Permission denied: failed to write to directory.");
                    panic!("Error occurred. Use `diff output.html expected.html` to see the problem.");
                }
            }
        )*
    };
}

test_filter!(

filter_comment: Filter::default().comment(true).text(false).tag(false).document(false) =>
"<!--@<li> --><!-- prettier-ignore --><!-- prettier-ignore --><!--- Table --->"

filter_doctype: Filter::default().document(true) =>
"<!><!DOCTYPE ><!DOCTYPE html>"

filter_text: Filter::default().text(true).tag(true) =>
r##"<html lang="en"><head><meta charset="UTF-8" /><meta name="viewport" content="width=device-width, initial-scale=1.0" /><title>Test HTML</title><style></style></head><body><header><h1>Test Page</h1><nav><ul><li><a xlink:href="#">About</a></li><li><a href="#">Contact<br> us</a></li></ul></nav></header><main class="container"><section><h2>Forms</h2><form action="#" method="post"><input type="sub\mit" id="name" name="name" /><input type='sub"mit' value="Submit" /><button enabled /><button enabled /></form></section><section><h2>Table</h2><table border="1"><thead><tr><th>ID</th><th>Name</th></tr></thead><tbody><tr><td>1</td><td>Alice</td></tr><tr><td>2</td><td>Bob</td></tr></tbody></table></section><section><h2>Lists</h2><ul><li>Item 1</li><li>Item 2</li></ul><ol><li>First</li><li>Second</li></ol></section><section><h2>Divs & Spans</h2><div class="box"></div><div class="box"></div><div class="box"></div><span>Inline span</span></section><section><h2>Media</h2><img src="test.jpg" alt="Test Image" /><video controls><source src="test.mp4" type="video /mp4" /></video></section><section><h2>Embedded Script</h2></section><section><h2>Forms with Various Inputs</h2><form><input type="checkbox" id="check" /><label for="check">Check me</label><input type="radio" name="radio" id="radio1" /><label for="radio1">Option 1</label><input type="radio" name="radio" id="radio2" /><label for="radio2">Option 2</label><input type="date" /><input type="file" /></form></section></main><footer><p>2025 Test Footer</p></footer></body></html>"##

);
