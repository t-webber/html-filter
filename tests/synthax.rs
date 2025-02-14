use html_parser::prelude::*;

#[test]
fn find_attr_value() {
    let html = parse_html(r#"<a id="std doc" enabled xlink:href="https://std.rs"/>"#).unwrap();

    if let Html::Tag { tag, .. } = html {
        assert!(tag.find_value("enabled").is_none());
        assert!(tag.find_value("xlink:href").map(|value| value.as_ref()) == Some("https://std.rs"));
        assert!(tag.as_name() == "a");
    } else {
        unreachable!()
    }
}
