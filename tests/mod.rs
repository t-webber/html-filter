use html_filter::prelude::{Filter, Html};

pub mod files;

#[test]
fn empty() {
    let html = Html::new();
    let filter = Filter::new();
    matches!(html.filter(&filter), Html::Empty);
}
