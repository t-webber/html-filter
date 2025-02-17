//! Module to define structs to filter

use super::NodeTypeFilter;
use super::element::{BlackWhiteList, ValueAssociateHash};
use crate::types::tag::Tag;

/// Filters to select the wanted elements of an Html tree.
///
/// The [`Filter`] structures allows you to
/// - remove some nodes: use the [`Self::comment`] (to remove all comments of
///   the form `<!-- comment -->`) or [`Self::doctype`] (to remove all doctype
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
/// Filter::new().comment(false).doctype(false); // Removes comments (`<!---->`) and doctype tags (`<!DOCTYPE html>`).
/// Filter::new().tag_name("a"); // Lists all the `<a>` tags and their content.
/// Filter::new().attribute_name("onclick"); // Lists all the tags with a `onclick` attribute.
/// Filter::new().attribute_value("id", "first-title"); // Get the element of `id` `"first-title`
/// Filter::new().tag_name("li").depth(1); // Lists all the `<li>` tags and their parent (usually `ol` or `ul`).
/// ```
#[non_exhaustive]
#[derive(Default, Debug)]
pub struct Filter {
    /// Attributes of tags
    ///
    /// This contains the list of attributes that ought to be kept in the final
    /// html tree, but also those that ought to be remove from the final.
    ///
    /// This includes attributes with or without values.
    attrs: ValueAssociateHash,
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
    /// This contains the list of tags that ought to be kept in the final html
    /// tree, but also those that ought to be remove from the final html.
    ///
    ///  # Examples
    ///
    /// `<a href="link" />`
    tags: BlackWhiteList,
    /// Filter by type of html node.
    types: NodeTypeFilter,
}

/// Private methods for [`Filter`]
impl Filter {
    /// Returns the wanted search depth
    pub(super) const fn as_depth(&self) -> usize {
        self.depth
    }

    /// Returns the types of nodes that must be kept according to the filter.
    pub(super) const fn as_types(&self) -> &NodeTypeFilter {
        &self.types
    }

    /// Checks if comments must be kept according to the filter.
    pub(super) const fn comment_allowed(&self) -> bool {
        self.types.comment_allowed()
    }

    /// Checks if doctypes must be kept according to the filter.
    pub(super) const fn doctype_allowed(&self) -> bool {
        self.types.doctype_allowed()
    }

    /// Checks if a given tag must be kept according to the filter..
    pub(super) fn tag_allowed(&self, tag: &Tag) -> bool {
        let name_allowed = self.tags.check(&tag.name);
        let attrs_allowed = self.attrs.check(&tag.attrs);
        let is_empty = self.attrs.is_empty() && self.tags.is_empty();
        name_allowed
            .and(&attrs_allowed)
            .is_explicitly_authorised(is_empty)
    }

    /// Checks if a given tag must be kept according to the filter..
    pub(super) fn tag_explicitly_allowed(&self, tag: &Tag) -> bool {
        let name_allowed = self.tags.check(&tag.name);
        let attrs_allowed = self.attrs.check(&tag.attrs);
        name_allowed
            .and(&attrs_allowed)
            .is_explicitly_authorised(false)
    }

    /// Checks if texts must be kept according to the filter.
    pub(super) const fn text_allowed(&self) -> bool {
        self.types.text_allowed()
    }
}

/// Public API for [`Filter`] on node-type-filters (texts, doctypes, comments,
/// etc.)
impl Filter {
    /// Removes the comments
    ///
    /// Doctypes and texts are kept, unless said otherwise by the user.
    #[inline]
    #[must_use]
    pub const fn all_except_comment(mut self) -> Self {
        self.types.set_only_comment(false);
        self
    }

    /// Removes the doctypes
    ///
    /// Comments and texts are kept, unless said otherwise by the user.
    #[inline]
    #[must_use]
    pub const fn all_except_doctype(mut self) -> Self {
        self.types.set_only_doctype(false);
        self
    }

    /// Removes the texts
    ///
    /// Comments and doctypes are kept, unless said otherwise by the user.
    #[inline]
    #[must_use]
    pub const fn all_except_text(mut self) -> Self {
        self.types.set_only_text(false);
        self
    }

    #[inline]
    #[must_use]
    /// Sets the filter for comments
    ///
    /// If `comment` is set to `true` (default), comments are kept.
    /// If `comment` is set to `false`, comments are removed.
    ///
    /// See [`Filter`] for usage information.
    pub const fn comment(mut self, comment: bool) -> Self {
        self.types.set_comment(comment);
        self
    }

    #[inline]
    #[must_use]
    /// Sets the filter for doctype tags
    ///
    /// If `doctype` is set to `true` (default), doctype tags are kept.
    /// If `doctype` is set to `false`, doctype tags are removed.
    ///
    /// See [`Filter`] for usage information.
    pub const fn doctype(mut self, doctype: bool) -> Self {
        self.types.set_doctype(doctype);
        self
    }

    /// Keeps only the comments
    ///
    /// Doctypes and texts are removed, unless said otherwise by the user.
    #[inline]
    #[must_use]
    pub const fn none_except_comment(mut self) -> Self {
        self.types.set_only_comment(true);
        self
    }

    /// Keeps only the doctypes
    ///
    /// Comments and texts are removed, unless said otherwise by the user.
    #[inline]
    #[must_use]
    pub const fn none_except_doctype(mut self) -> Self {
        self.types.set_only_doctype(true);
        self
    }

    /// Keeps only the texts
    ///
    /// Comments and doctypes are removed, unless said otherwise by the user.
    #[inline]
    #[must_use]
    pub const fn none_except_text(mut self) -> Self {
        self.types.set_only_text(true);
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
        self.types.set_text(text);
        self
    }
}

/// Public API for [`Filter`] on tags and attributes
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
        self.attrs.push(name.into(), None, true);
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
        self.attrs.push(name.into(), Some(value.into()), true);
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
    /// the comment, you must add `.comment(false)` to the filter):
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
    pub const fn depth(mut self, depth: usize) -> Self {
        self.depth = depth;
        self
    }

    #[inline]
    #[must_use]
    /// Specifies the name of an attribute in the tags that must be dismissed.
    ///
    /// This matches only tag attributes that don't have any value, such as
    /// `enabled` in
    ///
    /// ```html
    /// <button enabled type="submit" />
    /// ```
    ///
    /// See [`Filter`] for usage information.
    pub fn except_attribute_name<N: Into<String>>(mut self, name: N) -> Self {
        self.attrs.push(name.into(), None, false);
        self
    }

    #[inline]
    #[must_use]
    /// Specifies the value of an attribute in the tags that must be dismissed.
    ///
    /// This matches only tag attributes that have the correct value for the
    /// given name.
    ///
    /// See [`Filter`] for usage information.
    pub fn except_attribute_value<N, V>(mut self, name: N, value: V) -> Self
    where
        N: Into<String>,
        V: Into<String>,
    {
        self.attrs.push(name.into(), Some(value.into()), false);
        self
    }

    #[inline]
    #[must_use]
    /// Specifies the tag name of the wanted tags.
    ///
    /// See [`Filter`] for usage information.
    #[expect(unused_must_use, reason = "filter does not yet support results")]
    pub fn except_tag_name<N: Into<String>>(mut self, name: N) -> Self {
        self.tags.push(name.into(), false);
        self
    }

    /// Creates a default [`Filter`]
    ///
    /// By default, *comments* and *doctypes* are allowed, however no node is
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

    /// Disable all tags, except those explicitly whitelisted
    #[inline]
    #[must_use]
    pub const fn no_tags(mut self) -> Self {
        self.tags.set_default(false);
        self
    }

    #[inline]
    #[must_use]
    /// Specifies the tag name of the wanted tags.
    ///
    /// See [`Filter`] for usage information.
    #[expect(unused_must_use, reason = "filter does not yet support results")]
    pub fn tag_name<N: Into<String>>(mut self, name: N) -> Self {
        self.tags.push(name.into(), true);
        self
    }
}
