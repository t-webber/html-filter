//! Module to define structs to filter

use core::cmp::Ordering;
use std::collections::HashSet;

use crate::safe_expect;
use crate::types::html::Html;
use crate::types::tag::{Attribute, PrefixName, Tag};

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

/// Filters to select the wanted elements of an Html tree.
///
/// The [`Filter`] structures allows you to
/// - remove some nodes: use the [`Self::comment`] (to remove all comments of
///   the form `<!-- comment -->`) or [`Self::document`] (to remove all document
///   type nodes, such as `<!DOCTYPE html>`) methods.
/// - select some nodes, by searching them with their name (with the
///   [`Self::tag_name`] method) or attribute.s (with the
///   [`Self::attribute_name`] and [`Self::attribute_value`] methods).
/// - select those nodes and their parents, up to a certain generation (cf.
///   [`Self::depth`] method).
///
/// # Examples
///
/// ```
/// #![allow(unused)]
///
/// use html_parser::prelude::*;
///
/// Filter::new().comment(false).document(false); // Removes comments (`<!---->`) and document tags (`<!DOCTYPE html>`).
/// Filter::new().tag_name("a"); // Lists all the `<a>` tags and their content.
/// Filter::new().attribute_name("onclick"); // Lists all the tags with a `onclick` attribute.
/// Filter::new().attribute_value("id", "first-title"); // Get the element of `id` `"first-title`
/// Filter::new().tag_name("li").depth(1); // Lists all the `<li>` tags and their parent (usually `ol` or `ul`).
/// ```
#[non_exhaustive]
#[derive(Default, Debug)]
pub struct Filter {
    /// Attributes of the wanted tags
    attrs: Option<HashSet<Attribute>>,
    /// Depth in which to embed the required nodes
    ///
    /// # Examples
    ///
    /// If the html is `<nav><ul><li>Click on the <a
    /// href="#">link</a><li></ul></nav>` and we search with the filter
    ///
    /// ```
    /// use html_parser::prelude::*;
    ///
    /// let _filter = Filter::new().depth(1).tag_name("a");
    /// ```
    ///
    /// the expected output is `<li>Click on the <a href="#">link</a><li>`.
    ///
    /// - If the depth were 0, the output would have been only the `a` tag.
    /// - If the depth were 2, the output would have been the whole the `ul`
    ///   tag.
    depth: usize,
    /// Html tags
    ///
    ///  # Examples
    ///
    /// `<a href="link" />`
    tags: Option<HashSet<String>>,
    /// Filter by type of html node
    types: HtmlFilterType,
}

impl Filter {
    /// Method to check all the attributes are present.
    fn allowed_tag(&self, tag: &Tag) -> bool {
        match (self.tags.as_ref(), self.attrs.as_ref()) {
            (None, None) => false,
            (tags, attrs) =>
                tags.is_none_or(|names| names.contains(&tag.name))
                    && attrs.is_none_or(|wanted| wanted.iter().all(|attr| tag.attrs.contains(attr))),
        }
    }

    #[inline]
    #[must_use]
    /// Specifies the name of an attribute in the wanted tags.
    ///
    /// This matches only tag attributes that don't have any value, such as
    /// `enabled` in
    ///
    /// ```html
    /// <button enabled type="submit" />
    /// ```
    ///
    /// See [`Filter`] for usage information.
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
    /// Specifies the value of an attribute in the wanted tags.
    ///
    /// This matches only tag attributes that have the correct value for the
    /// given name.
    ///
    /// See [`Filter`] for usage information.
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

    #[inline]
    #[must_use]
    /// Filters comments
    ///
    /// If `comment` is set to `true` (default), comments are kept.
    /// If `comment` is set to `false`, comments are removed.
    ///
    /// See [`Filter`] for usage information.
    pub const fn comment(mut self, comment: bool) -> Self {
        self.types.comment = comment;
        self
    }

    #[inline]
    #[must_use]
    /// Specifies the depth of the desired nodes.
    ///
    /// The *depth* means at what depth the nodes must be kept for this node.
    /// This allows you to search for a node, and select the node, but also some
    /// of its ancestors, up to the chosen depth. For instance, a depth of 0
    /// means you only keep the tag, but a depth of 1 means you keep the wanted
    /// tag, but it's parent and all its children.
    ///
    /// # Examples
    ///
    /// For example, let's consider this HTML code:
    ///
    /// ```html
    /// <main>
    ///     <nav>
    ///         <!-- Navigation menu -->
    ///         <ul>
    ///             <li href="first">First link</li>
    ///             <li href="second">Second link</li>
    ///             <li href="third">Third link</li>
    ///         </ul>
    ///     </nav>
    /// </main>
    /// ```
    ///
    /// For this piece of HTML code, the filter
    ///
    /// ```
    /// #![allow(unused)]
    /// html_parser::filter::Filter::new()
    ///     .attribute_value("href", "second")
    ///     .depth(0);
    /// ```
    ///
    /// will return:
    ///
    /// ```html
    /// <li href="second">Second link</li>
    /// ```
    ///
    /// ;
    ///
    /// ```
    /// #![allow(unused)]
    /// html_parser::filter::Filter::new()
    ///     .attribute_value("href", "second")
    ///     .depth(1);
    /// ```
    ///
    /// will return (note that the other children were kept):
    ///
    /// ```html
    /// <ul>
    ///     <li href="first">First link</li>
    ///     <li href="second">Second link</li>
    ///     <li href="third">Third link</li>
    /// </ul>
    /// ```
    ///
    /// ;
    ///
    /// ```
    /// #![allow(unused)]
    /// html_parser::filter::Filter::new()
    ///     .attribute_value("href", "second")
    ///     .depth(2);
    /// ```
    ///
    /// will return (note that even the comment was kept, if you want to remove
    /// the comment, you must add `.comment(false` to the filter):
    ///
    /// ```html
    /// <nav>
    ///     <!-- Navigation menu -->
    ///     <ul>
    ///         <li href="first">First link</li>
    ///         <li href="second">Second link</li>
    ///         <li href="third">Third link</li>
    ///     </ul>
    /// </nav>
    /// ```
    //TODO: c'est vrai ca que pour enlever on met commentaire ?
    pub const fn depth(mut self, depth: usize) -> Self {
        self.depth = depth;
        self
    }

    #[inline]
    #[must_use]
    /// Filters document-style tags
    ///
    /// A document-style tag is a tag that starts with an exclamation mark, such
    /// as `<!DOCTYPE html>`.
    ///
    /// If `document` is set to `true` (default), document-style tags are kept.
    /// If `document` is set to `false`, document-style tags are removed.
    ///
    /// See [`Filter`] for usage information.
    pub const fn document(mut self, document: bool) -> Self {
        self.types.document = document;
        self
    }

    /// Creates a default [`Filter`]
    ///
    /// By default, *comments* and *documents* are allowed, however no node is
    /// wanted, so filtering on a default filter will return an empty [`Html`].
    ///
    /// # Examples
    ///
    /// ```
    /// use html_parser::prelude::*;
    ///
    /// let _filter: Filter = Filter::new();
    /// ```
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    #[must_use]
    /// Specifies the tag name of the wanted tags.
    ///
    /// See [`Filter`] for usage information.
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
    /// Creates a [`FilterSuccess`] from an [`Html`]
    #[expect(clippy::unnecessary_wraps, reason = "useful for filter method")]
    const fn found(html: Html) -> Option<Self> {
        Some(Self { depth: DepthSuccess::Found(0), html })
    }

    /// Increment the depth, if applicable
    #[inline]
    #[expect(clippy::unnecessary_wraps, reason = "useful for filter method")]
    fn incr(mut self) -> Option<Self> {
        self.depth = self.depth.incr();
        Some(self)
    }
}

impl Html {
    /// Method to check if a wanted node is visible
    ///
    /// This methods stop checking after a maximum depth, as the current node
    /// will be discarded if it is deeper in the tree.
    // TODO: users can implement this an be disapointed
    fn check_depth(&self, max_depth: usize, filter: &Filter) -> Option<usize> {
        match self {
            Self::Empty | Self::Text(_) => None,
            Self::Comment { .. } => filter.types.comment.then_some(0),
            Self::Document { .. } => filter.types.document.then_some(0),
            Self::Tag { tag, .. } if filter.allowed_tag(tag) => Some(0),
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
    /// See [`Filter`] to know how to define a filter.
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
        self.filter_aux(filter).html
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
    #[expect(clippy::ref_patterns, reason = "ref only on one branch")]
    #[expect(
        clippy::arithmetic_side_effects,
        reason = "incr depth when smaller than filter_depth"
    )]
    fn filter_aux(self, filter: &Filter) -> FilterSuccess {
        let output = match self {
            Self::Comment { .. } if !filter.types.comment => None,
            Self::Document { .. } if !filter.types.document => None,
            Self::Text(txt) if txt.chars().all(char::is_whitespace) => None,

            Self::Tag { ref tag, .. } if filter.allowed_tag(tag) => FilterSuccess::found(self),
            Self::Tag { child, .. } if filter.depth == 0 => child.filter_aux(filter).incr(),
            Self::Tag { child, tag, full } => {
                let rec = child.filter_aux(filter);
                match rec.depth {
                    DepthSuccess::None => None,
                    DepthSuccess::Success => Some(rec),
                    DepthSuccess::Found(depth) => match depth.cmp(&filter.depth) {
                        Ordering::Less => Some(FilterSuccess {
                            depth: DepthSuccess::Found(depth + 1),
                            html: Self::Tag { tag, full, child: Box::new(rec.html) },
                        }),
                        Ordering::Equal | Ordering::Greater =>
                            Some(FilterSuccess { depth: DepthSuccess::Success, html: rec.html }),
                    },
                }
            }

            Self::Vec(vec) => {
                match vec
                    .iter()
                    .filter_map(|child| child.check_depth(filter.depth + 1, filter))
                    .collect::<Vec<_>>()
                    .iter()
                    .min()
                {
                    Some(depth) if *depth < filter.depth => Some(FilterSuccess {
                        depth: DepthSuccess::Found(*depth),
                        html: Self::Vec(vec),
                    }),
                    Some(_) => Some(FilterSuccess {
                        depth: DepthSuccess::Success,
                        html: Self::Vec(
                            vec.into_iter()
                                .map(|child| child.filter_aux(filter))
                                .filter(|child| !child.html.is_empty())
                                .map(|child| child.html)
                                .collect::<Vec<_>>(),
                        ),
                    }),
                    None => {
                        let mut filtered = vec
                            .into_iter()
                            .map(|child| child.filter_aux(filter))
                            .filter(|node| !node.html.is_empty())
                            .collect::<Vec<FilterSuccess>>();
                        if filtered.len() <= 1 {
                            filtered.pop()
                        } else {
                            filtered.iter().map(|child| child.depth).min().map(|depth| {
                                FilterSuccess {
                                    depth,
                                    html: Self::Vec(
                                        filtered.into_iter().map(|child| child.html).collect(),
                                    ),
                                }
                            })
                        }
                    }
                }
            }

            Self::Text(_) | Self::Empty => None,
            Self::Comment { .. } | Self::Document { .. } => FilterSuccess::found(self),
        }
        .unwrap_or_default();
        output
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
