//! Module to filter an HTML tree to keep or remove specific nodes, with a set
//! of rules.
//!
//! You can either filter your HTML with [`Html::filter`] or find a specific
//! node with [`Html::find`].
//!
//! For more information on how to define the filtering rules, please refer to
//! [`Filter`].

extern crate alloc;
mod element;
mod node_type;
pub mod types;

use alloc::borrow::Cow;
use core::cmp::Ordering;

use node_type::NodeTypeFilter;
use types::Filter;

use crate::errors::{safe_expect, safe_unreachable};
use crate::prelude::{Html, Tag};

/// State to follow if the wanted nodes where found at what depth
///
/// # Note
///
/// We implement the discriminant and specify the representation size in order
/// to derive [`Ord`] trait.
#[repr(u8)]
#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
enum DepthSuccess {
    /// Wanted node wanting more depth
    Found(usize) = 1,
    /// Not wanted node, doesn't respect the filters
    #[default]
    None = 2,
    /// Wanted node with already the wanted depth
    Success = 0,
}

impl DepthSuccess {
    /// Increment the depth, if applicable
    fn incr(mut self) -> Self {
        if let Self::Found(depth) = &mut self {
            *depth = safe_expect!(depth.checked_add(1), "Smaller than required depth");
        }

        self
    }
}

/// Status of the filtering on recursion calls
#[derive(Default, Debug)]
struct FilterSuccess {
    /// Indicates if the filter found a wanted node
    ///
    /// Is
    /// - `None` if no wanted node was found
    /// - `Some(depth)` if a wanted node was found at depth `depth`. If there
    ///   are embedded nodes that satisfy the filter, `depth` is the smallest
    ///   possible.
    depth: DepthSuccess,
    /// Result of the filtering
    html: Html,
}

impl FilterSuccess {
    /// Increment the depth, if applicable
    #[expect(clippy::unnecessary_wraps, reason = "useful for filter method")]
    fn incr(mut self) -> Option<Self> {
        self.depth = self.depth.incr();
        Some(self)
    }

    /// Creates a [`FilterSuccess`] from an [`Html`]
    ///
    /// This is the method to use when the node is considered `found`, i.e.,
    /// when it was the node the user was looking for.
    #[expect(clippy::unnecessary_wraps, reason = "useful for filter method")]
    const fn make_found(html: Html) -> Option<Self> {
        Some(Self { depth: DepthSuccess::Found(0), html })
    }

    /// Creates a [`FilterSuccess`] from an [`Html`]
    ///
    /// This is the method to use when the node isn't interesting alone, it can
    /// be if it is in the right scope though.
    #[expect(clippy::unnecessary_wraps, reason = "useful for filter method")]
    fn make_none(html: Cow<'_, Html>) -> Option<Self> {
        Some(Self { depth: DepthSuccess::None, html: html.into_owned() })
    }
}

impl Html {
    /// Method to check if a wanted node is visible
    ///
    /// This methods stop checking after a maximum depth, as the current node
    /// will be discarded if it is deeper in the tree.
    fn check_depth(&self, max_depth: usize, filter: &Filter) -> Option<usize> {
        match self {
            Self::Empty | Self::Text(_) | Self::Comment { .. } | Self::Doctype { .. } => None,
            Self::Tag { tag, .. } if filter.tag_explicitly_allowed(tag) => Some(0),
            Self::Tag { .. } | Self::Vec(_) if max_depth == 0 => None,
            Self::Tag { child, .. } => child
                .check_depth(
                    #[expect(clippy::arithmetic_side_effects, reason = "non-0")]
                    {
                        max_depth - 1
                    },
                    filter,
                )
                .map(
                    #[expect(clippy::arithmetic_side_effects, reason = "< initial max_depth")]
                    |depth| depth + 1,
                ),
            Self::Vec(vec) => vec
                .iter()
                .try_fold(Some(usize::MAX), |acc, child| {
                    if acc == Some(0) {
                        Err(())
                    } else {
                        Ok(child.check_depth(max_depth, filter))
                    }
                })
                .unwrap_or(Some(0)),
        }
    }

    /// Filters html based on a defined filter.
    ///
    /// See [`Filter`] to learn how to create filters.
    ///
    /// Filters allow you to select the portions of the html code you want to
    /// keep or remove.
    ///
    /// # Returns
    ///
    /// The html tree obtains by keeping only the nodes that fulfills the
    /// filter.
    #[must_use]
    pub fn filter(self, filter: &Filter) -> Self {
        filter_aux(Cow::Owned(self), filter, false).html
    }

    /// Finds an html node based on a defined filter.
    ///
    /// See [`Filter`] to know how to define a filter.
    ///
    /// Filters allow you to select the portions of the html code you want to
    /// keep or remove.
    ///
    /// # Returns
    ///
    /// The first node that fulfills the filter.
    #[must_use]
    pub fn find(self, filter: &Filter) -> Self {
        self.filter(filter).into_first()
    }

    /// Keeps only the first element of a filtered output
    fn into_first(self) -> Self {
        if let Self::Vec(vec) = self {
            for elt in vec {
                let res = elt.into_first();
                if !res.is_empty() {
                    return res;
                }
            }
            safe_unreachable("Filtering removes empty nodes in vec.")
        } else {
            self
        }
    }

    /// Filters html based on a defined filter.
    ///
    /// Equivalent of [`Html::filter`] when data is not owned.
    #[must_use]
    pub fn to_filtered(&self, filter: &Filter) -> Self {
        filter_aux(Cow::Borrowed(self), filter, false).html
    }

    /// Finds an html node based on a defined filter.
    ///
    /// Equivalent of [`Html::find`] when data is not owned.
    //TODO: data except first is cloned
    #[must_use]
    pub fn to_found(&self, filter: &Filter) -> Self {
        self.to_filtered(filter).into_first()
    }
}

/// Wrapper for [`Html::filter`]
///
/// Refer to [`Html::filter`] for documentation.
///
/// This methods takes an additional `clean` boolean to indicate when a tag
/// returns the child. In that case, the texts must disappear if present at
/// root.
///
/// This methods returns a wrapper of the final html in a [`FilterSuccess`]
/// to follow the current depth of the last found node. See
/// [`FilterSuccess`] for more information.
#[allow(clippy::allow_attributes, reason = "expect is buggy")]
#[allow(
    clippy::enum_glob_use,
    reason = "heavy syntax and Html is the main struct"
)]
fn filter_aux(cow_html: Cow<'_, Html>, filter: &Filter, found: bool) -> FilterSuccess {
    use Html::*;
    match cow_html {
        Cow::Borrowed(Comment(_)) | Cow::Owned(Comment(_))
            if found || !filter.comment_explicitly_allowed() =>
            None,
        Cow::Borrowed(Doctype { .. }) | Cow::Owned(Doctype { .. })
            if found || !filter.doctype_allowed() =>
            None,
        Cow::Borrowed(Doctype { .. } | Comment(_)) | Cow::Owned(Doctype { .. } | Comment(_)) =>
            FilterSuccess::make_none(cow_html),
        Cow::Borrowed(Text(_) | Empty) | Cow::Owned(Text(_) | Empty) => None,
        Cow::Borrowed(Tag { tag, child }) =>
            filter_aux_tag(Cow::Borrowed(&**child), Cow::Borrowed(tag), filter, found),
        Cow::Owned(Tag { tag, child }) =>
            filter_aux_tag(Cow::Owned(*child), Cow::Owned(tag), filter, found),
        Cow::Borrowed(Vec(vec)) => filter_aux_vec(Cow::Borrowed(vec), filter),
        Cow::Owned(Vec(vec)) => filter_aux_vec(Cow::Owned(vec), filter),
    }
    .unwrap_or_default()
}

/// Auxiliary method for [`filter_aux`] on [`Html::Tag`]
#[expect(
    clippy::arithmetic_side_effects,
    reason = "incr depth when smaller than filter_depth"
)]
fn filter_aux_tag(
    child: Cow<'_, Html>,
    tag: Cow<'_, Tag>,
    filter: &Filter,
    found: bool,
) -> Option<FilterSuccess> {
    if filter.tag_allowed(tag.as_ref()) {
        FilterSuccess::make_found(Html::Tag {
            tag: tag.into_owned(),
            child: Box::new(filter_light(child, filter)),
        })
    } else if filter.as_depth() == 0 {
        filter_aux(child, filter, found).incr()
    } else {
        let rec = filter_aux(child, filter, found);
        match rec.depth {
            DepthSuccess::None => None,
            DepthSuccess::Success => Some(rec),
            DepthSuccess::Found(depth) => match depth.cmp(&filter.as_depth()) {
                Ordering::Less => Some(FilterSuccess {
                    depth: DepthSuccess::Found(depth + 1),
                    html: Html::Tag { tag: tag.into_owned(), child: Box::new(rec.html) },
                }),
                Ordering::Equal | Ordering::Greater =>
                    Some(FilterSuccess { depth: DepthSuccess::Success, html: rec.html }),
            },
        }
    }
}

/// Auxiliary method for [`filter_aux`] on [`Html::Vec`]
#[expect(
    clippy::arithmetic_side_effects,
    reason = "incr depth when smaller than filter_depth"
)]
fn filter_aux_vec(vec: Cow<'_, Box<[Html]>>, filter: &Filter) -> Option<FilterSuccess> {
    match vec
        .as_ref()
        .iter()
        .filter_map(|child| child.check_depth(filter.as_depth() + 1, filter))
        .min()
    {
        Some(depth) if depth < filter.as_depth() => Some(FilterSuccess {
            depth: DepthSuccess::Found(depth),
            html: Html::Vec(
                vec.iter()
                    .map(|child| filter_light(Cow::Borrowed(child), filter))
                    .collect(),
            ),
        }),
        Some(_) => Some(FilterSuccess {
            depth: DepthSuccess::Success,
            html: Html::Vec(into_iter_filter_map_collect(vec, |child| {
                let rec = filter_aux(child, filter, true);
                if rec.html.is_empty() {
                    None
                } else {
                    Some(rec.html)
                }
            })),
        }),
        None => {
            let mut filtered: Vec<FilterSuccess> = into_iter_filter_map_collect(vec, |child| {
                let rec = filter_aux(child, filter, false);
                if rec.html.is_empty() { None } else { Some(rec) }
            });
            if filtered.len() <= 1 {
                filtered.pop()
            } else {
                filtered
                    .iter()
                    .map(|child| child.depth)
                    .min()
                    .map(|depth| FilterSuccess {
                        depth,
                        html: Html::Vec(filtered.into_iter().map(|child| child.html).collect()),
                    })
            }
        }
    }
}

/// Light filter without complicated logic, just filtering on types.
///
/// This method does take into account the [`Filter::tag_name`],
///   [`Filter::attribute_name`] and [`Filter::attribute_value`] methods,
/// only the types of [`NodeTypeFilter`].
///
/// The return type is [`Html`] and not [`Cow`] has it is only called on
/// successes.
#[allow(clippy::allow_attributes, reason = "expect is buggy")]
#[allow(
    clippy::enum_glob_use,
    reason = "heavy syntax and Html is the main struct"
)]
fn filter_light(cow_html: Cow<'_, Html>, filter: &Filter) -> Html {
    use Html::*;
    match cow_html {
        Cow::Borrowed(Text(_)) | Cow::Owned(Text(_)) if filter.text_allowed() =>
            cow_html.into_owned(),
        Cow::Borrowed(Comment(_)) | Cow::Owned(Comment(_)) if filter.comment_allowed() =>
            cow_html.into_owned(),
        Cow::Borrowed(Doctype { .. }) | Cow::Owned(Doctype { .. }) if filter.doctype_allowed() =>
            cow_html.into_owned(),
        Cow::Borrowed(Tag { tag, .. }) if filter.tag_explicitly_blacklisted(tag) => Html::Empty,
        Cow::Owned(Tag { tag, .. }) if filter.tag_explicitly_blacklisted(&tag) => Html::Empty,
        Cow::Borrowed(Tag { tag, child }) => Tag {
            tag: tag.to_owned(),
            child: Box::new(filter_light(Cow::Borrowed(&**child), filter)),
        },
        Cow::Owned(Tag { tag, child }) =>
            Tag { tag, child: Box::new(filter_light(Cow::Owned(*child), filter)) },
        Cow::Borrowed(Vec(vec)) => Html::Vec(
            vec.into_iter()
                .map(|child| filter_light(Cow::Borrowed(child), filter))
                .collect(),
        ),
        Cow::Owned(Vec(vec)) => Html::Vec(
            vec.into_iter()
                .map(|child| filter_light(Cow::Owned(child), filter))
                .collect(),
        ),
        Cow::Borrowed(Empty | Text(_) | Comment { .. } | Doctype { .. })
        | Cow::Owned(Empty | Text(_) | Comment { .. } | Doctype { .. }) => Html::Empty,
    }
}

/// Method to apply [`Iterator::filter_map`] on an iterator inside a Cow,
/// without losing the Cow.
pub fn into_iter_filter_map_collect<T, U, V, F>(cow: Cow<'_, Box<[T]>>, map: F) -> V
where
    T: Clone,
    V: FromIterator<U>,
    F: Fn(Cow<'_, T>) -> Option<U>,
{
    match cow {
        Cow::Borrowed(borrowed) => borrowed
            .into_iter()
            .filter_map(|elt| map(Cow::Borrowed(elt)))
            .collect(),
        Cow::Owned(owned) => owned
            .into_iter()
            .filter_map(|elt| map(Cow::Owned(elt)))
            .collect(),
    }
}
