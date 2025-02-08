use std::fs::{self, read_to_string};

use html_parser::filter::Filter;
use html_parser::parse::parse_html;

fn format_html(html: &str) -> String {
    let mut old = html
        .replace('/', " /")
        .replace('\n', " ")
        .replace("< ", "<")
        .replace(" >", ">")
        .replace("<", " <")
        .replace(">", "> ");
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
                let tree = parse_html(&content).unwrap_or_else(|err| panic!("{err}")).filter_html(&$filter);
                let formatted_input = format_html(&$expect);
                let formatted_output = format_html(&format!("{tree}"));
                if formatted_output != formatted_input {
                    let output_path = format!("output.{}.html", stringify!($name));
                    let expected_path = format!("expected.{}.html", stringify!($name));
                    fs::write(&output_path, &formatted_output)
                        .expect("Permission denied: failed to write to directory.");
                    fs::write(&expected_path, &formatted_input)
                        .expect("Permission denied: failed to write to directory.");
                    panic!("Error occurred.\nOutput:\n--------------------\n{formatted_output}\n--------------------\nUse `diff {output_path} {expected_path}` to see the problem.");
                }
            }
        )*
    };
}

test_filter!(

filter_comment: Filter::default().comment(true).document(false) =>
"<!--@<li> --><!-- prettier-ignore --><!-- prettier-ignore --><!--- Table --->"

filter_doctype: Filter::default().document(true) =>
"<!><!DOCTYPE ><!DOCTYPE html>"

filter_prefix: Filter::default().attribute_value("xlink:href", "#") =>
r##"<a xlink:href="#">About</a>"##

filter_radio: Filter::default().attribute_value("type", "radio").attribute_name("radio") =>
r#"<input radio type="radio" name="radio" id="radio1" /><input radio type="radio" name="radio" id="radio2" />"#

find_radio: Filter::default().attribute_value("type", "radio").attribute_value("id", "radio2") =>
r#"<input radio type="radio" name="radio" id="radio2" />"#

filter_enabled: Filter::default().attribute_name("enabled") =>
"<button enabled /><input enabled />"

filter_buttons: Filter::default().tag_name("button").tag_name("input") =>
r#"
<input type="sub\mit" id="name" name="name" />
<input type='sub"mit' value="Submit" />
<button enabled />
<input enabled />
<input type="checkbox" id="check" />
<input radio type="radio" name="radio" id="radio1" />
<input radio type="radio" name="radio" id="radio2" />
<input type="date" />
<input type="file" />
"#

filter_tr: Filter::default().tag_name("tr") =>
"<tr><th>ID</th><th>Name</th></tr><tr><td>1</td><td>Alice</td></tr><tr><td>2</td><td>Bob</td></tr>"

);
