//! Module to define structs to filter

use std::collections::HashSet;

use crate::types::html::Html;
use crate::types::tag::{Attribute, PrefixName, Tag};

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
    /// Html tags
    ///
    ///  # Examples
    ///
    /// `<a href="link" />`
    tags: Option<HashSet<String>>,
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
    fn allowed_tag(&self, tag: &Tag) -> bool {
        self.tags
            .as_ref()
            .is_some_and(|names| names.contains(&tag.name))
            || self
                .attrs
                .as_ref()
                .is_some_and(|wanted| wanted.iter().all(|attr| tag.attrs.contains(attr)))
    }
    #[inline]
    #[must_use]
    /// Activates everything, except if tag names or attributes were given.
    pub const fn all(mut self) -> Self {
        self.types.comment = true;
        self.types.document = true;
        self
    }

    filter_setter!(comment document);

    #[inline]
    #[must_use]
    /// Adds a required attribute in the selected tags.
    pub fn tag_name<N: Into<String>>(mut self, name: N) -> Self {
        if let Some(names) = &mut self.tags {
            names.insert(name.into());
        } else {
            let mut names = HashSet::new();
            names.insert(name.into());
            self.tags = Some(names);
        }
        self
    }
}

impl Html {
    /// Filters html based on a defined filter.
    #[inline]
    #[must_use]
    pub fn filter(self, filter: &Filter) -> Self {
        self.filter_aux(filter, false)
    }

    /// Wrapper for [`Self::filter`]
    ///
    /// It takes an additional `clean` boolean to indicate when a tag returns
    /// the child, the texts must disappear.
    #[expect(clippy::ref_patterns, reason = "ref only on one branch")]
    fn filter_aux(self, filter: &Filter, clean: bool) -> Self {
        match self {
            Self::Comment { .. } if !filter.types.comment => Self::default(),
            Self::Document { .. } if !filter.types.document => Self::default(),
            Self::Text(txt) if txt.chars().all(char::is_whitespace) => Self::default(),

            Self::Tag { ref tag, .. } if filter.allowed_tag(tag) => self,
            Self::Tag { child, .. } => child.filter_aux(filter, true),
            Self::Vec(vec) => {
                let mut filtered_vec = Vec::with_capacity(vec.len());
                for child in vec {
                    let filtered_child = child.filter(filter);
                    if !filtered_child.is_empty()
                        && (!clean || !matches!(filtered_child, Self::Text(_)))
                    {
                        filtered_vec.push(filtered_child);
                    }
                }
                if filtered_vec.len() <= 1 {
                    filtered_vec.pop().unwrap_or_default()
                } else {
                    Self::Vec(filtered_vec)
                }
            }
            Self::Comment { .. } | Self::Document { .. } | Self::Empty | Self::Text(_) => self,
        }
    }

    /// Filters html based on a defined filter.
    #[inline]
    #[must_use]
    #[expect(clippy::ref_patterns, reason = "ref only on one branch")]
    pub fn find(self, filter: &Filter) -> Option<Self> {
        match self {
            Self::Comment { .. } if !filter.types.comment => None,
            Self::Document { .. } if !filter.types.document => None,
            Self::Text(txt) if txt.chars().all(char::is_whitespace) => None,

            Self::Tag { ref tag, .. } if filter.allowed_tag(tag) => Some(self),
            Self::Tag { child, .. } => child.find(filter),
            Self::Vec(vec) => {
                for child in vec {
                    if let Some(found) = child.find(filter) {
                        return Some(found);
                    }
                }
                None
            }
            Self::Comment { .. } | Self::Document { .. } | Self::Empty | Self::Text(_) => None,
        }
    }
}

/// Types of html nodes to filter
///
/// Set the elements to `true` iff you want them to appear in the filtered
/// output
#[derive(Default, Debug)]
struct HtmlFilterType {
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
}
