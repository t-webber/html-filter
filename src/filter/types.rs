//! Module to define structs to filter

use std::collections::HashSet;

use crate::types::tag::{Attribute, PrefixName, Tag};

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
    /// - If the depth were `0`, the output would have been only the `a` tag.
    /// - If the depth were `2`, the output would have been the whole the `ul`
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
    /// Returns the wanted search depth
    pub(super) const fn as_depth(&self) -> usize {
        self.depth
    }

    /// Returns the types of nodes that must be kept according to the filter.
    pub(super) const fn as_types(&self) -> &HtmlFilterType {
        &self.types
    }

    /// Checks if comments must be kept according to the filter.
    pub(super) const fn comment_allowed(&self) -> bool {
        self.types.comment
    }

    /// Checks if doctypes must be kept according to the filter.
    pub(super) const fn document_allowed(&self) -> bool {
        self.types.document
    }

    /// Checks if a given tag must be kept according to the filter..
    pub(super) fn tag_allowed(&self, tag: &Tag) -> bool {
        let exception = match (self.tags.as_ref(), self.attrs.as_ref()) {
            (None, None) => false,
            (tags, attrs) =>
                tags.is_none_or(|names| names.contains(&tag.name))
                    && attrs.is_none_or(|wanted| wanted.iter().all(|attr| tag.attrs.contains(attr))),
        };
        // self.types.tag ^ exception
        exception
    }

    /// Checks if texts must be kept according to the filter.
    pub(super) const fn text_allowed(&self) -> bool {
        self.types.text
    }
}

impl Filter {
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
    /// The *depth* means at what depth the nodes must be kept according to the
    /// filter. for this node. This allows you to search for a node, and
    /// select the node, but also some of its ancestors, up to the chosen
    /// depth. For instance, a depth of 0 means you only keep the tag, but a
    /// depth of 1 means you keep the wanted tag, but it's parent and all
    /// its children.
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
    /// html_parser::prelude::Filter::new()
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
    /// html_parser::prelude::Filter::new()
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
    /// html_parser::prelude::Filter::new()
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
    /// wanted, so filtering on a default filter will return an empty
    /// [`Html`](super::Html).
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

    #[inline]
    #[must_use]
    /// Filters texts
    ///
    /// - If `text` is set to `true` (default), all texts are kept.
    /// - If `text` is set to `false`, all texts are removed.
    ///
    /// See [`Filter`] for usage information.
    pub const fn text(mut self, text: bool) -> Self {
        self.types.text = text;
        self
    }
}

/// Types of html nodes to filter
///
/// Set the elements to `true` iff you want them to appear in the filtered
/// output
#[derive(Default, Debug)]
pub(super) struct HtmlFilterType {
    /// Html comment
    ///
    /// # Examples
    ///
    /// `<!-- some comment -->`
    pub comment: bool,
    /// Html document tags
    ///
    /// # Examples
    ///
    /// `<!-- some comment -->`
    pub document: bool,
    // /// Html tags
    // ///
    // /// # Examples
    // ///
    // /// `<p>`, `<a />` or `<br>`
    // tag: bool,
    /// Html text node
    ///
    /// # Examples
    ///
    /// In `<p>Hello world</p>`, `Hello world` is a text node.
    pub text: bool,
}
