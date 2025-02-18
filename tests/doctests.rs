//! This file is not meant for manual editing.
//! This file is automatically generated by `build.rs` script.
//! Any changes made will be discarded.

#[test]
fn auto_doctest_1() {
    // Auto generated from src/parse/mod.rs:18
    use html_parser::prelude::*;
    let html: &str = r#"
    <!DOCTYPE html>
    <html lang="en">
        <head>
            <title>Html sample</title>
        </head>
        <body>
            <p>This is an html sample.</p>
        </body>
    </html>
    "#;
    let tree: Html = parse_html(html).expect("Invalid HTML");
    assert!(format!("{tree}") == html);
}

#[test]
fn auto_doctest_2() {
    // Auto generated from src/types/html.rs:20
    use html_parser::prelude::*;
    let _html: Html = parse_html(
        r#"<nav>
        <!-- Navigation menu -->
        <ul>
            <li href="first">First link</li>
            <li href="second">Second link</li>
            <li href="third">Third link</li>
        </ul>
    </nav>"#,
    )
    .unwrap();
}

#[test]
fn auto_doctest_3() {
    // Auto generated from src/types/tag.rs:216
    use html_parser::prelude::*;
    let html = parse_html("<a enabled href='https://crates.io'>").unwrap();
    if let Html::Tag { tag, .. } = html {
        assert!(tag.as_name() == "a");
        assert!(tag.find_attr_value("enabled").is_none());
        assert!(
            tag.find_attr_value("href")
                .is_some_and(|value| value == "https://crates.io")
        );
        let value: String = tag.into_attr_value("href").unwrap();
        assert!(&value == "https://crates.io");
    } else {
        unreachable!();
    }
}

#[test]
fn auto_doctest_4() {
    // Auto generated from src/types/tag.rs:256
    use html_parser::prelude::*;
    let html = parse_html("<div />").unwrap();
    if let Html::Tag { tag, .. } = html {
        assert!(tag.as_name() == "div");
    } else {
        unreachable!();
    }
}

#[test]
fn auto_doctest_5() {
    // Auto generated from src/types/tag.rs:281
    use html_parser::prelude::*;
    let html = parse_html(r#"<a id="std doc" enabled xlink:href="https://std.rs"/>"#).unwrap();
    if let Html::Tag { tag, .. } = html {
        assert!(tag.find_attr_value("enabled").is_none());
        assert!(
            tag.find_attr_value("xlink:href")
                .map(|value| value.as_ref())
                == Some("https://std.rs")
        );
    } else {
        unreachable!()
    }
}

#[test]
fn auto_doctest_6() {
    // Auto generated from src/types/tag.rs:316
    use html_parser::prelude::*;
    let html = parse_html(r#"<a enabled/>"#).unwrap();
    if let Html::Tag { tag, .. } = html {
        assert!(tag.into_attr_value("enabled").is_none());
    } else {
        unreachable!()
    }
    let html = parse_html(r#"<a id="std doc" href="https://std.rs"/>"#).unwrap();
    if let Html::Tag { tag, .. } = html {
        assert!(
            tag.into_attr_value("href")
                .is_some_and(|value| &value == "https://std.rs")
        );
    } else {
        unreachable!()
    }
}

#[test]
fn auto_doctest_7() {
    // Auto generated from src/filter/types.rs:21
    #![allow(unused)]
    use html_parser::prelude::*;
    Filter::new().comment(false).doctype(false); // Removes comments (`<!---->`) and doctype tags (`<!DOCTYPE html>`).
    Filter::new().tag_name("a"); // Lists all the `<a>` tags and their content.
    Filter::new().attribute_name("onclick"); // Lists all the tags with a `onclick` attribute.
    Filter::new().attribute_value("id", "first-title"); // Get the element of `id` `"first-title`
    Filter::new().tag_name("li").depth(1); // Lists all the `<li>` tags and their parent (usually `ol` or `ul`).
}

#[test]
fn auto_doctest_8() {
    // Auto generated from src/filter/types.rs:49
    use html_parser::prelude::*;
    let _filter = Filter::new().depth(1).tag_name("a");
}

#[test]
fn auto_doctest_9() {
    // Auto generated from src/filter/types.rs:299
    #![allow(unused)]
    html_parser::prelude::Filter::new()
        .attribute_value("href", "second")
        .depth(0);
}

#[test]
fn auto_doctest_10() {
    // Auto generated from src/filter/types.rs:314
    #![allow(unused)]
    html_parser::prelude::Filter::new()
        .attribute_value("href", "second")
        .depth(1);
}

#[test]
fn auto_doctest_11() {
    // Auto generated from src/filter/types.rs:333
    #![allow(unused)]
    html_parser::prelude::Filter::new()
        .attribute_value("href", "second")
        .depth(2);
}

#[test]
fn auto_doctest_12() {
    // Auto generated from src/filter/types.rs:411
    use html_parser::prelude::*;
    let _filter: Filter = Filter::new();
}
