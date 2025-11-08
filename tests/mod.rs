use html_filter::{Filter, Html};

pub mod files;

#[test]
fn empty() {
    let html = Html::new();
    let filter = Filter::new();
    matches!(html.filter(&filter), Html::Empty);
}
