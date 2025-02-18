//! Module to filter an HTML tree to keep or remove specific nodes, with a set
//! of rules.
//!
//! You can either filter your HTML with [`Html::filter`] or find a specific
//! node with [`Html::find`].
//!
//! For more information on how to define the filtering rules, please refer to
//! [`Filter`].

mod element;
mod node_type;
pub mod types;

use core::cmp::Ordering;

use node_type::NodeTypeFilter;
use types::Filter;

use crate::errors::safe_unreachable;
use crate::prelude::Tag;
use crate::safe_expect;
use crate::types::html::Html;
use crate::types::tag::TagType;

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
    #[inline]
    #[coverage(off)]
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
    #[inline]
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
    const fn make_none(html: Html) -> Option<Self> {
        Some(Self { depth: DepthSuccess::None, html })
    }
}

impl Html {
    /// Method to check if a wanted node is visible
    ///
    /// This methods stop checking after a maximum depth, as the current node
    /// will be discarded if it is deeper in the tree.
    fn check_depth(&self, max_depth: usize, filter: &Filter) -> Option<usize> {
        // let input = format!("{self:?}").chars().take(200).collect::<String>();
        let output = match self {
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
        };
        //         println!(
        //             "
        // ~~~~~~~~~~~~~~~~~~~~~~~~
        // {input}

        // =>
        // {output:?}

        // ~~~~~~~~~~~~~~~~~~~~~~~~
        //      "
        //         );
        output
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
    #[inline]
    #[must_use]
    pub fn filter(self, filter: &Filter) -> Self {
        self.filter_aux(filter, false).html
    }

    /// Wrapper for [`Self::filter`]
    ///
    /// Refer to [`Self::filter`] for documentation.
    ///
    /// This methods takes an additional `clean` boolean to indicate when a tag
    /// returns the child. In that case, the texts must disappear if present at
    /// root.
    ///
    /// This methods returns a wrapper of the final html in a [`FilterSuccess`]
    /// to follow the current depth of the last found node. See
    /// [`FilterSuccess`] for more information.
    fn filter_aux(self, filter: &Filter, found: bool) -> FilterSuccess {
        // let input = format!(
        //     "{self:?}
        // "
        // )
        // .chars()
        // .take(150)
        // .collect::<String>();
        let output = match self {
            Self::Comment { .. } if found || !filter.comment_explicitly_allowed() => None,
            Self::Doctype { .. } if found || !filter.doctype_allowed() => None,
            Self::Text(txt)
                if found || !filter.text_allowed() || txt.chars().all(char::is_whitespace) =>
                None,

            Self::Tag { tag, child, full } =>
                Self::filter_aux_tag(*child, tag, full, filter, found),
            Self::Vec(vec) => Self::filter_aux_vec(vec, filter),

            Self::Text(_) | Self::Empty => None,
            Self::Comment { .. } | Self::Doctype { .. } => FilterSuccess::make_none(self),
        }
        .unwrap_or_default();
        //         println!(
        //             "
        // ------------------------------------------------------
        // {input}

        // =>
        // {:?}
        // \n{}

        // ------------------------------------------------------
        //                              ",
        //             output.depth, output.html
        //         );
        output
    }

    /// Auxiliary method for [`Self::filter_aux`] on [`Html::Vec`]
    #[expect(
        clippy::arithmetic_side_effects,
        reason = "incr depth when smaller than filter_depth"
    )]
    fn filter_aux_tag(
        child: Self,
        tag: Tag,
        full: TagType,
        filter: &Filter,
        found: bool,
    ) -> Option<FilterSuccess> {
        if filter.tag_allowed(&tag) {
            FilterSuccess::make_found(Self::Tag {
                tag,
                full,
                child: Box::new(child.filter_light(filter)),
            })
        } else if filter.as_depth() == 0 {
            child.filter_aux(filter, found).incr()
        } else {
            let rec = child.filter_aux(filter, found);
            match rec.depth {
                DepthSuccess::None => None,
                DepthSuccess::Success => Some(rec),
                DepthSuccess::Found(depth) => match depth.cmp(&filter.as_depth()) {
                    Ordering::Less => Some(FilterSuccess {
                        depth: DepthSuccess::Found(depth + 1),
                        html: Self::Tag { tag, full, child: Box::new(rec.html) },
                    }),
                    Ordering::Equal | Ordering::Greater =>
                        Some(FilterSuccess { depth: DepthSuccess::Success, html: rec.html }),
                },
            }
        }
    }

    /// Auxiliary method for [`Self::filter_aux`] on [`Html::Vec`]
    #[expect(
        clippy::arithmetic_side_effects,
        reason = "incr depth when smaller than filter_depth"
    )]
    fn filter_aux_vec(vec: Vec<Self>, filter: &Filter) -> Option<FilterSuccess> {
        match vec
            .iter()
            .filter_map(|child| child.check_depth(filter.as_depth() + 1, filter))
            .collect::<Vec<_>>()
            .iter()
            .min()
        {
            Some(depth) if *depth < filter.as_depth() => Some(FilterSuccess {
                depth: DepthSuccess::Found(*depth),
                html: Self::Vec(
                    vec.into_iter()
                        .map(|child| child.filter_light(filter))
                        .collect(),
                ),
            }),
            Some(_) => Some(FilterSuccess {
                depth: DepthSuccess::Success,
                html: Self::Vec(
                    vec.into_iter()
                        .map(|child| child.filter_aux(filter, true))
                        .filter(|child| !child.html.is_empty())
                        .map(|child| child.html)
                        .collect::<Vec<_>>(),
                ),
            }),
            None => {
                let mut filtered = vec
                    .into_iter()
                    .map(|child| child.filter_aux(filter, false))
                    .filter(|node| {
                        !node.html.is_empty() // && node.depth.successful(filter.as_depth())
                    })
                    .collect::<Vec<_>>();
                if filtered.len() <= 1 {
                    filtered.pop()
                } else {
                    filtered
                        .iter()
                        .map(|child| child.depth)
                        .min()
                        .map(|depth| FilterSuccess {
                            depth,
                            html: Self::Vec(filtered.into_iter().map(|child| child.html).collect()),
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
    #[coverage(off)]
    fn filter_light(self, filter: &Filter) -> Self {
        match self {
            Self::Text(_) if filter.text_allowed() => self,
            Self::Comment { .. } if filter.comment_allowed() => self,
            Self::Doctype { .. } if filter.doctype_allowed() => self,
            Self::Tag { tag, full, child } =>
                Self::Tag { tag, full, child: Box::new(child.filter_light(filter)) },
            Self::Vec(vec) => Self::Vec(
                vec.into_iter()
                    .map(|child| child.filter_light(filter))
                    .collect(),
            ),
            Self::Empty | Self::Text(_) | Self::Comment { .. } | Self::Doctype { .. } =>
                Self::Empty,
        }
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
    #[inline]
    #[must_use]
    pub fn find(self, filter: &Filter) -> Self {
        self.filter(filter).into_first()
    }

    /// Keeps only the first element of a filtered output
    #[coverage(off)]
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
}
