//! Module to define structs to filter

use core::mem::take;
use std::collections::HashSet;

use crate::types::html::Html;
use crate::types::tag::{Attribute, PrefixName};

/// Macro to setup a filter
macro_rules! filter_setter {
    ($($name:ident)*) => {
       $(
            #[doc = concat!("Activates the ", stringify!($name), "s in the filter")]
            #[inline]
            #[must_use]
            /// Activates the specified field for filtering.
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
    /// Attributes of the wanted tags
    attrs: Option<HashSet<Attribute>>,
    /// Filter by type of html node
    types: HtmlFilterType,
}

#[expect(clippy::arbitrary_source_item_ordering, reason = "macro used")]
impl Filter {
    #[inline]
    #[must_use]
    /// Adds a required attribute in the selected tags.
    pub fn attribute_name<N: Into<String>>(mut self, name: N) -> Self {
        let attr = Attribute::NameNoValue(PrefixName::from(name.into()));
        if let Some(attrs) = &mut self.attrs {
            attrs.insert(attr);
        } else {
            let mut hash_set = HashSet::new();
            hash_set.insert(attr);
            self.attrs = Some(hash_set);
        }
        self
    }

    #[inline]
    #[must_use]
    /// Adds a required attribute in the selected tags.
    pub fn attribute_value<N: Into<String>, V: Into<String>>(mut self, name: N, value: V) -> Self {
        let attr = Attribute::NameValue {
            name: PrefixName::from(name.into()),
            value: value.into(),
            double_quote: true,
        };
        if let Some(attrs) = &mut self.attrs {
            attrs.insert(attr);
        } else {
            let mut hash_set = HashSet::new();
            hash_set.insert(attr);
            self.attrs = Some(hash_set);
        }
        self
    }

    /// Method to check all the attributes are present.
    fn compare_attrs(&self, found: &[Attribute]) -> bool {
        self.attrs
            .as_ref()
            .is_some_and(|wanted| wanted.iter().all(|attr| found.contains(attr)))
    }

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
        Html::Tag { tag, .. } if filter.compare_attrs(&tag.attrs) => (),
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
