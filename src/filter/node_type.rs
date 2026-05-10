//! Types of html nodes to filter
//!
//! Set the elements to `true` iff you want them to appear in the filtered
//! output

/// Types of html nodes to filter
///
/// Set the elements to `true` iff you want them to appear in the filtered
/// output
#[derive(Default, Debug, PartialEq, Eq)]
pub(super) struct NodeTypeFilter {
    /// Whether successive texts after a filter removes nodes should be collapse
    /// in one text or not.
    collapse: bool,
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
    /// Whether to trim all texts.
    ///
    /// This will remove text segments that contain only whitespaces and
    /// newlines.
    trim: bool,
}

#[expect(clippy::arbitrary_source_item_ordering, reason = "ordered by type")]
impl NodeTypeFilter {
    // helpers

    /// Returns a default [`Self`]
    pub const fn new() -> Self {
        Self { comment: None, doctype: None, text: None, trim: false, collapse: false }
    }

    // getters

    /// Returns whether the texts should be collapse or not after filtering.
    pub const fn as_collapse(&self) -> bool {
        self.collapse
    }

    /// Checks if comments are allowed
    pub const fn comment_allowed(&self) -> Option<bool> {
        self.comment
    }

    /// Checks if doctypes are allowed
    pub const fn doctype_allowed(&self) -> Option<bool> {
        self.doctype
    }

    /// Checks if texts are allowed
    pub const fn text_allowed(&self) -> Option<bool> {
        self.text
    }

    /// Checks if texts should be trimmed, and removed if empty.
    pub const fn should_trim(&self) -> bool {
        self.trim
    }

    // setters

    /// Collapses successive text nodes.
    pub const fn set_collapse(&mut self) {
        self.collapse = true;
    }

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

    /// Sets trim flag.
    pub const fn trim(&mut self) {
        self.trim = true;
    }
}
