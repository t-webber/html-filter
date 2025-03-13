use std::fs::read_to_string;

use html_filter::prelude::*;

use super::test_maker;

macro_rules! test_filter {
    ($($name:ident: $filter:expr => $expect:expr)*) => {
        $(
            #[test]
            fn $name() {
                let content = read_to_string("tests/data/index.html").unwrap();
                let tree = parse_html(&content).unwrap_or_else(|err| panic!("{err}"));
                let filtered_cloned = (&tree).to_filtered(&$filter);
                let filtered = tree.filter(&$filter);
                test_maker(stringify!($name), $expect, filtered_cloned, $filter);
                test_maker(stringify!($name), $expect, filtered, $filter);
            }
        )*
    };
}

test_filter!(

doctype: Filter::new().no_tags().none_except_doctype() =>
"<!><!DOCTYPE ><!DOCTYPE html>"

prefix: Filter::new().attribute_value("xlink:href", "#").none_except_text() =>
r##"<a xlink:href="#">About</a>"##

radio: Filter::new().attribute_value("type", "radio").attribute_name("radio") =>
r#"<input radio type="radio" name="radio" id="radio1" /><input radio type="radio" name="radio" id="radio2" />"#

radio_id: Filter::new().attribute_value("type", "radio").attribute_value("id", "radio2") =>
r#"<input radio type="radio" name="radio" id="radio2" />"#

radio_id_except: Filter::new().attribute_value("type", "radio").except_attribute_value("id", "radio2") =>
r#"<input radio type="radio" name="radio" id="radio1" />"#

enabled: Filter::new().attribute_name("enabled") =>
"<button enabled /><input enabled />"

input_enabled: Filter::new().attribute_name("enabled").except_tag_name("button") =>
"<input enabled />"

button_enabled: Filter::new().except_tag_name("button").tag_name("button").attribute_name("enabled") =>
"<button enabled />"

buttons: Filter::new().tag_name("button").tag_name("input") =>
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

non_radio_input: Filter::new().except_attribute_name("radio").tag_name("input") =>
r#"
<input type="sub\mit" id="name" name="name" />
<input type='sub"mit' value="Submit" />
<input enabled />
<input type="checkbox" id="check" />
<input type="date" />
<input type="file" />
"#

tr: Filter::new().tag_name("tr").comment(false) =>
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

tag: Filter::new().tag_name("form").attribute_value("action", "#").comment(false) =>
r##"
<form action="#" method="post">
    <input type="sub\mit" id="name" name="name" />
    <input type='sub"mit' value="Submit" />
    <button enabled/>
</form>
"##

depth_multiple: Filter::new().depth(1).attribute_name("enabled") =>
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

depth_multiple_no_text: Filter::new().depth(1).attribute_name("enabled").text(false) =>
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

depth_multiple_no_text_no_submit: Filter::new().depth(1).attribute_name("enabled").text(false).except_attribute_value("value", "Submit") =>
r##"
<form action="#" method="post">
    <input type="sub\mit" id="name" name="name" />
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

depth_with_comment: Filter::new().depth(1).attribute_value("border", "1").comment(true) =>
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

depth_no_comment: Filter::new().depth(1).attribute_value("border", "1").comment(false) =>
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

no_script_style:     Filter::new().except_tag_name("script").except_tag_name("style") =>
r##"
<!>
<!DOCTYPE >
<!DOCTYPE html>
<html lang="en">
    <head>
        <meta charset="UTF-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1.0" />
        <title>Test HTML</title>
    </head>
    <body>
        <header>
            <h1>Test Page</h1>
            <nav>
                <ul>
                    <!--@<li> -->
                    <li><a xlink:href="#">About</a></li>
                    <li>
                        <!-- prettier-ignore -->
                        <a href="#">Contact<br> us</a>
                    </li>
                </ul>
            </nav>
        </header>
        <main class="container">
            <section>
                <h2>Forms</h2>
                <form action="#" method="post">
                    <input type="sub\mit" id="name" name="name" />
                    <input type='sub"mit' value="Submit" />
                    <!-- prettier-ignore -->
                    <button enabled/>
                </form>
            </section>

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

            <section>
                <h2>Divs & Spans</h2>
                <div class="box"></div>
                <div class="box"></div>
                <div class="box"></div>
                <span>Inline span</span>
            </section>

            <section>
                <h2>Media</h2>
                <img src="test.jpg" alt="Test Image" />
                <video controls>
                    <source src="test.mp4" type="video/mp4" />
                </video>
            </section>

            <section>
                <h2>Embedded Script</h2>
            </section>

            <section>
                <h2>Forms with Various Inputs</h2>
                <form>
                    <input type="checkbox" id="check" />
                    <label for="check">Check me</label>
                    <input radio type="radio" name="radio" id="radio1" />
                    <label for="radio1">Option 1</label>
                    <input radio type="radio" name="radio" id="radio2" />
                    <label for="radio2">Option 2</label>
                    <input type="date" />
                    <input type="file" />
                </form>
            </section>
        </main>

        <footer>
            <p>2025 Test Footer</p>
        </footer>

    </body>
</html>
"##

);
