//! Module to define structs to filter

use core::mem::take;

use crate::types::html::Html;

/// Macro to setup a filter
macro_rules! filter_setter {
    ($($name:ident)*) => {
       $(
            #[doc = concat!("Activates the ", stringify!($name), "s in the filter")]
            #[inline]
            #[must_use]
            pub const fn $name(mut self, $name: bool) -> Self {
                self.types.$name = $name;
                self
            }
        )*
    };
}

/// Data structure to defines the filters to select the wanted elements of the
/// Html tree
#[non_exhaustive]
#[derive(Default, Debug)]
pub struct Filter {
    /// Filter by type of html node
    types: HtmlFilterType,
}

impl Filter {
    filter_setter!(comment document tag text);
}

/// Types of html nodes to filter
///
/// Set the elements to `true` iff you want them to appear in the filtered
/// output
#[non_exhaustive]
#[expect(clippy::struct_excessive_bools, reason = "not a state machine")]
#[derive(Default, Debug)]
pub struct HtmlFilterType {
    /// Html comment
    ///
    /// # Examples
    ///
    /// `<!-- some comment -->`
    comment: bool,
    /// Html document tags
    ///
    /// # Examples
    ///
    /// `<!-- some comment -->`
    document: bool,
    /// Html tags
    ///
    ///  # Examples
    ///
    /// `<a href="link" />`
    tag: bool,
    /// Html texts
    ///
    ///  # Examples
    ///
    /// `some text`
    text: bool,
}

/// Filters html based on a defined filter.
#[inline]
pub fn filter_html(html: &mut Html, filter: &Filter) {
    match html {
        Html::Comment { .. } if !filter.types.comment => *html = Html::default(),
        Html::Document { .. } if !filter.types.document => {
            *html = Html::default();
        }
        Html::Text(txt) if !filter.types.text || txt.chars().all(char::is_whitespace) =>
            *html = Html::default(),
        Html::Tag { child, .. } => {
            filter_html(child, filter);
            if !filter.types.tag {
                *html = take(child);
            }
        }
        Html::Vec(vec) => {
            for mut child in take(vec) {
                filter_html(&mut child, filter);
                if !child.is_empty() {
                    vec.push(child);
                }
            }
            if vec.len() <= 1 {
                if let Some(last) = vec.pop() {
                    *html = last;
                } else {
                    *html = Html::default();
                }
            }
        }
        Html::Comment { .. } | Html::Document { .. } | Html::Empty | Html::Text(_) => (), //
    }
}
