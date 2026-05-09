//! Tests to check the crate works as expected.
#![cfg(test)]
#![expect(clippy::panic, clippy::expect_used, reason = "tests")]

/// Inner module to prevent the tests from being parsing without going through
/// this file.
pub mod files;

use html_filter::{Filter, Html};

#[test]
fn empty() {
    let html = Html::new();
    let filter = Filter::new();
    matches!(html.filter(&filter), Html::Empty);
}

#[test]
fn new_default() {
    assert_eq!(format!("{:?}", Filter::new()), format!("{:?}", Filter::default()));
}

const _CONST_FILTER: Filter = Filter::new();
