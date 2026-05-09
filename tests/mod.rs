use html_filter::{Filter, Html};

pub mod files;

#[test]
fn empty() {
    let html = Html::new();
    let filter = Filter::new();
    matches!(html.filter(&filter), Html::Empty);
}

const _CONST_FILTER: Filter = Filter::new();
