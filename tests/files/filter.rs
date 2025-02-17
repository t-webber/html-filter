use std::fs::read_to_string;

use html_parser::prelude::*;

use super::test_maker;

macro_rules! test_filter {
    ($($name:ident: $filter:expr => $expect:expr)*) => {
        $(
            #[test]
            fn $name() {
                let content = read_to_string("tests/data/index.html").unwrap();
                let tree = parse_html(&content).unwrap_or_else(|err| panic!("{err}")).filter(&$filter);
                test_maker(stringify!($name), $expect, tree, $filter)
            }
        )*
    };
}

test_filter!(

filter_doctype: Filter::new().no_tags().none_except_doctype() =>
"<!><!DOCTYPE ><!DOCTYPE html>"

filter_prefix: Filter::new().attribute_value("xlink:href", "#").none_except_text() =>
r##"<a xlink:href="#">About</a>"##

filter_radio: Filter::new().attribute_value("type", "radio").attribute_name("radio") =>
r#"<input radio type="radio" name="radio" id="radio1" /><input radio type="radio" name="radio" id="radio2" />"#

filter_radio_id: Filter::new().attribute_value("type", "radio").attribute_value("id", "radio2") =>
r#"<input radio type="radio" name="radio" id="radio2" />"#

filter_radio_id_except: Filter::new().attribute_value("type", "radio").except_attribute_value("id", "radio2") =>
r#"<input radio type="radio" name="radio" id="radio1" />"#

filter_enabled: Filter::new().attribute_name("enabled") =>
"<button enabled /><input enabled />"

filter_input_enabled: Filter::new().attribute_name("enabled").except_tag_name("button") =>
"<input enabled />"

filter_button_enabled: Filter::new().except_tag_name("button").tag_name("button").attribute_name("enabled") =>
"<button enabled />"

filter_buttons: Filter::new().tag_name("button").tag_name("input") =>
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

filter_non_radio_input: Filter::new().except_attribute_name("radio").tag_name("input") =>
r#"
<input type="sub\mit" id="name" name="name" />
<input type='sub"mit' value="Submit" />
<input enabled />
<input type="checkbox" id="check" />
<input type="date" />
<input type="file" />
"#

filter_tr: Filter::new().tag_name("tr").text(true) =>
"<tr><th>ID</th><th>Name</th></tr><tr><td>1</td><td>Alice</td></tr><tr><td>2</td><td>Bob</td></tr>"

depth_1: Filter::new().depth(1).tag_name("source") =>
r##"
<video controls>
    <source src="test.mp4" type="video/mp4" />
</video>
"##

depth_2: Filter::new().depth(2).tag_name("source") =>
r##"
<section>
    <h2>Media</h2>
    <img src="test.jpg" alt="Test Image" />
    <video controls>
        <source src="test.mp4" type="video/mp4" />
    </video>
</section>
"##

filter_tag: Filter::new().tag_name("form").attribute_value("action", "#") =>
r##"
<form action="#" method="post">
    <input type="sub\mit" id="name" name="name" />
    <input type='sub"mit' value="Submit" />
    <button enabled/>
</form>
"##

depth_multiple: Filter::new().depth(1).attribute_name("enabled").comment(true) =>
r##"
<form action="#" method="post">
    <input type="sub\mit" id="name" name="name" />
    <input type='sub"mit' value="Submit" />
    <!-- prettier-ignore -->
    <button enabled/>
</form>
<section>
    <h2>Lists</h2>
    <ul>
        <li>Item 1</li>
        <li>Item 2</li>
    </ul>
    <ol>
        <li>First</li>
        <li>Second</li>
    </ol>
    <input enabled />
</section>
"##

depth_multiple_no_text: Filter::new().depth(1).attribute_name("enabled").none_except_comment() =>
r##"
<form action="#" method="post">
    <input type="sub\mit" id="name" name="name" />
    <input type='sub"mit' value="Submit" />
    <!-- prettier-ignore -->
    <button enabled/>
</form>
<section>
    <h2></h2>
    <ul>
        <li></li>
        <li></li>
    </ul>
    <ol>
        <li></li>
        <li></li>
    </ol>
    <input enabled />
</section>
"##

filter_depth_with_comment: Filter::new().depth(1).attribute_value("border", "1").comment(true) =>
r##"
<section>
    <h2><!--- Table --->Table</h2>
    <table border="1">
        <thead>
            <tr>
                <th>ID</th>
                <th>Name</th>
            </tr>
        </thead>
        <tbody>
            <tr>
                <td>1</td>
                <td>Alice</td>
            </tr>
            <tr>
                <td>2</td>
                <td>Bob</td>
            </tr>
        </tbody>
    </table>
</section>
"##

filter_depth_no_comment: Filter::new().depth(1).attribute_value("border", "1").comment(false) =>
r##"
<section>
    <h2>Table</h2>
    <table border="1">
        <thead>
            <tr>
                <th>ID</th>
                <th>Name</th>
            </tr>
        </thead>
        <tbody>
            <tr>
                <td>1</td>
                <td>Alice</td>
            </tr>
            <tr>
                <td>2</td>
                <td>Bob</td>
            </tr>
        </tbody>
    </table>
</section>
"##

);
