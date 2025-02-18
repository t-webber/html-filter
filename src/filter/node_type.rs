//! Types of html nodes to filter
//!
//! Set the elements to `true` iff you want them to appear in the filtered
//! output

use crate::unwrap_or;

/// Types of html nodes to filter
///
/// Set the elements to `true` iff you want them to appear in the filtered
/// output
#[derive(Default, Debug)]
pub(super) struct NodeTypeFilter {
    /// Html comment
    ///
    /// # Note
    ///
    /// By default, comments are discarded.
    ///
    /// # Examples
    ///
    /// `<!-- some comment -->`
    comment: Option<bool>,
    /// Html doctype tags
    ///
    /// # Note
    ///
    /// By default, doctype are discarded.
    ///
    /// # Examples
    ///
    /// `<!-- some comment -->`
    doctype: Option<bool>,
    /// Html text node
    ///
    /// # Note
    ///
    /// By default, texts are kept.
    ///
    /// # Examples
    ///
    /// In `<p>Hello world</p>`, `Hello world` is a text node.
    text: Option<bool>,
}

#[expect(clippy::arbitrary_source_item_ordering, reason = "ordered by type")]
impl NodeTypeFilter {
    /* getters */

    /// Checks if comments are allowed
    pub const fn comment_allowed(&self) -> Option<bool> {
        self.comment
    }
    /// Checks if doctypes are allowed
    pub const fn doctype_allowed(&self) -> Option<bool> {
        self.doctype
    }
    /// Checks if texts are allowed
    pub const fn text_allowed(&self) -> bool {
        unwrap_or(self.text, true)
    }

    /* setters */

    /// Sets the comment authorisation
    pub const fn set_comment(&mut self, comment: bool) {
        self.comment = Some(comment);
    }
    /// Sets the doctype authorisation
    pub const fn set_doctype(&mut self, doctype: bool) {
        self.doctype = Some(doctype);
    }
    /// Sets the text authorisation
    pub const fn set_text(&mut self, text: bool) {
        self.text = Some(text);
    }

    /// Sets the comment authorisation, and all others to the contrary if not
    /// already set.
    pub const fn set_only_comment(&mut self, comment: bool) {
        self.set_all(!comment);
        self.comment = Some(comment);
    }
    /// Sets the doctype authorisation, and all others to the contrary if not
    /// already set.
    pub const fn set_only_doctype(&mut self, doctype: bool) {
        self.set_all(!doctype);
        self.doctype = Some(doctype);
    }
    /// Sets the text authorisation, and all others to the contrary if not
    /// already set.
    pub const fn set_only_text(&mut self, text: bool) {
        self.set_all(!text);
        self.text = Some(text);
    }

    /// Sets all the types to the same value, if not already set.
    const fn set_all(&mut self, keep: bool) {
        if self.comment.is_none() {
            self.comment = Some(keep);
        }
        if self.doctype.is_none() {
            self.doctype = Some(keep);
        }
        if self.text.is_none() {
            self.text = Some(keep);
        }
    }
}
