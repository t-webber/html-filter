use html_filter::{Filter, Html};

use crate::files::strings::INPUT;
use crate::files::test_maker;

#[test]
fn trim() {
    test_maker(
        "trim",
        "<!DOCTYPE html><!-- comment 1 --><html>A first text<!-- first comment \
         --><p>A<strong>first</strong>text<!-- second comment --><img></img></p></html>",
        &Html::parse(INPUT).expect("failed to parse").filter(&Filter::new().trim()),
        "",
        false,
    );
}

#[test]
fn trim_simple() {
    test_maker(
        "trim",
        "<div>A div with spaces</div>",
        &Html::parse(" <div> A div with spaces </div> ")
            .expect("failed to parse")
            .filter(&Filter::new().trim()),
        "",
        false,
    );
}

#[test]
fn trim_simple_borrowed() {
    test_maker(
        "trim",
        "<div>A div with spaces</div>",
        &Html::parse(" <div> A div with spaces </div> ")
            .expect("failed to parse")
            .to_filtered(&Filter::new().trim()),
        "",
        false,
    );
}

#[test]
fn remove_empty() {
    let filter = Filter::new().tag_name("tr").trim();
    let html = Html::parse("<tr>\n<th></th>\n</tr>\n").expect("failed to parse").filter(&filter);
    let (tag, child) = html.as_tag().expect("not a tag");
    assert_eq!(tag.name, "tr");
    let (th_tag, th_child) = child.as_tag().expect("not a tag");
    assert_eq!(th_tag.name, "th");
    assert_eq!(*th_child, Html::Empty);
}
