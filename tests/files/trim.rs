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
