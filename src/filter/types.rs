//! Module to define structs to filter

use super::NodeTypeFilter;
use super::element::{BlackWhiteList, ValueAssociateHash};
use crate::types::tag::Tag;
use crate::unwrap_or;

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
/// use html_filter::*;
///
/// Filter::new().comment(false).doctype(false); // Removes comments (`<!---->`) and doctype tags (`<!DOCTYPE html>`).
/// Filter::new().tag_name("a"); // Lists all the `<a>` tags and their content.
/// Filter::new().attribute_name("onClick"); // Lists all the tags with a `onClick` attribute.
/// Filter::new().attribute_value("id", "first-title"); // Get the element of `id` `"first-title`
/// Filter::new().tag_name("li").depth(1); // Lists all the `<li>` tags and their parent (usually `ol` or `ul`).
/// Filter::new().none_except_text().collapse().trim().no_tags(); // Returns text without padding
///                                                               // between tags and in one Html::Text
/// ```
#[non_exhaustive]
#[derive(Default, Debug)]
#[expect(clippy::field_scoped_visibility_modifiers, reason = "useless")]
pub struct Filter {
    /// Attributes of tags
    ///
    /// This contains the list of attributes that ought to be kept in the final
    /// html tree, but also those that ought to be remove from the final.
    ///
    /// This includes attributes with or without values.
    pub(super) attrs: ValueAssociateHash,
    /// Depth in which to embed the required nodes
    ///
    /// # Examples
    ///
    /// If the html is `<nav><ul><li>Click on the <a
    /// href="#">link</a><li></ul></nav>` and we search with the filter
    ///
    /// ```
    /// use html_filter::*;
    ///
    /// let _filter = Filter::new().depth(1).tag_name("a");
    /// ```
    ///
    /// the expected output is `<li>Click on the <a href="#">link</a><li>`.
    ///
    /// - If the depth were `0`, the output would have been only the `a` tag.
    /// - If the depth were `2`, the output would have been the whole the `ul`
    ///   tag.
    pub(super) depth: usize,
    /// Html tags
    ///
    /// This contains the list of tags that ought to be kept in the final html
    /// tree, but also those that ought to be remove from the final html.
    ///
    ///  # Examples
    ///
    /// `<a href="link" />`
    pub(super) tags: BlackWhiteList,
    /// Filter by type of html node.
    pub(super) types: NodeTypeFilter,
}

/// Private methods for [`Filter`]
impl Filter {
    /// Checks whethers the texts should be collapsed or not after filtering.
    pub(super) const fn as_collapse(&self) -> bool {
        self.types.as_collapse()
    }

    /// Returns the wanted search depth
    pub(super) const fn as_depth(&self) -> usize {
        self.depth
    }

    /// Checks if comments must be kept according to the filter.
    pub(super) const fn comment_allowed(&self) -> bool {
        unwrap_or(self.types.comment_allowed(), true)
    }

    /// Checks if comments must be kept according to the filter.
    pub(super) const fn comment_explicitly_allowed(&self) -> bool {
        unwrap_or(self.types.comment_allowed(), self.is_empty())
    }

    /// Checks if doctypes must be kept according to the filter.
    pub(super) const fn doctype_allowed(&self) -> bool {
        unwrap_or(self.types.doctype_allowed(), self.is_empty())
    }

    /// Checks if no rules were given concerning tags and attributes
    const fn is_empty(&self) -> bool {
        self.tags.is_empty() && self.attrs.is_empty()
    }

    /// Checks if texts should be trimmed, and removed if empty.
    pub(super) const fn should_trim(&self) -> bool {
        self.types.should_trim()
    }

    /// Checks if a given tag must be kept according to the filter
    pub(super) fn tag_allowed(&self, tag: &Tag) -> bool {
        let name_allowed = self.tags.check(tag.as_name());
        let attrs_allowed = self.attrs.check(tag.as_attrs());
        name_allowed.and(&attrs_allowed).is_allowed_or(self.is_empty())
    }

    /// Checks if a given tag has an explicit rule, rule to keep this tag
    pub(super) fn tag_explicitly_allowed(&self, tag: &Tag) -> bool {
        let name_allowed = self.tags.check(tag.as_name());
        let attrs_allowed = self.attrs.check(tag.as_attrs());
        name_allowed.and(&attrs_allowed).is_allowed_or(false)
    }

    /// Checks if a given tag has an explicit rule, rule to keep this tag
    pub(super) fn tag_explicitly_blacklisted(&self, tag: &Tag) -> bool {
        self.tags.is_explicitly_blacklisted(tag.as_name())
            || self.attrs.is_explicitly_blacklisted(tag.as_attrs())
    }

    /// Checks if texts must be kept according to the filter
    pub(super) const fn text_allowed(&self) -> bool {
        unwrap_or(self.types.text_allowed(), true)
    }

    /// Checks if comments must be kept according to the filter.
    pub(super) const fn text_explicitly_allowed(&self) -> bool {
        unwrap_or(self.types.text_allowed(), self.is_empty())
    }
}
