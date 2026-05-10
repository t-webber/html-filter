//! Public API for [`Filter`]

use crate::Filter;
use crate::filter::NodeTypeFilter;
use crate::filter::element::{AttributeMatch, BlackWhiteList, ValueAssociateHash};

/// Public API for [`Filter`] on node-type-filters (texts, doctypes, comments,
/// etc.)
impl Filter {
    /// Short-hand to set the keep policy of comments, texts and doctypes at
    /// once.
    ///
    /// - `true`: keep them
    /// - `false`: remove them
    ///
    /// It is equivalent to:
    ///
    /// ```
    /// use html_filter::*;
    /// assert_eq!(Filter::new().doctype(true).text(true).comment(true), Filter::new().all(true));
    /// assert_eq!(Filter::new().doctype(false).text(false).comment(false), Filter::new().all(false));
    /// ```
    #[must_use]
    pub const fn all(self, all: bool) -> Self {
        self.comment(all).doctype(all).text(all)
    }

    /// Removes the comments
    ///
    /// Doctypes and texts are kept, unless said otherwise by the user.
    #[must_use]
    pub const fn all_except_comment(self) -> Self {
        self.all(true).comment(false)
    }

    /// Removes the doctypes
    ///
    /// Comments and texts are kept, unless said otherwise by the user.
    #[must_use]
    pub const fn all_except_doctype(self) -> Self {
        self.all(true).doctype(false)
    }

    /// Removes the texts
    ///
    /// Comments and doctypes are kept, unless said otherwise by the user.
    #[must_use]
    pub const fn all_except_text(self) -> Self {
        self.all(true).text(false)
    }

    /// Sets the filter for comments
    ///
    /// If `comment` is set to `true` (default), comments are kept.
    /// If `comment` is set to `false`, comments are removed.
    ///
    /// See [`Filter`] for usage information.
    #[must_use]
    pub const fn comment(mut self, comment: bool) -> Self {
        self.types.set_comment(comment);
        self
    }

    /// Sets the filter for doctype tags
    ///
    /// If `doctype` is set to `true` (default), doctype tags are kept.
    /// If `doctype` is set to `false`, doctype tags are removed.
    ///
    /// See [`Filter`] for usage information.
    #[must_use]
    pub const fn doctype(mut self, doctype: bool) -> Self {
        self.types.set_doctype(doctype);
        self
    }

    /// Keeps only the comments
    ///
    /// Doctypes and texts are removed, unless said otherwise by the user.
    #[must_use]
    pub const fn none_except_comment(self) -> Self {
        self.all(false).comment(true)
    }

    /// Keeps only the doctypes
    ///
    /// Comments and texts are removed, unless said otherwise by the user.
    #[must_use]
    pub const fn none_except_doctype(self) -> Self {
        self.all(false).doctype(true)
    }

    /// Keeps only the texts
    ///
    /// Comments and doctypes are removed, unless said otherwise by the user.
    #[must_use]
    pub const fn none_except_text(self) -> Self {
        self.all(false).text(true)
    }

    /// Filters texts
    ///
    /// - If `text` is set to `true` (default), all texts are kept.
    /// - If `text` is set to `false`, all texts are removed.
    ///
    /// See [`Filter`] for usage information.
    #[must_use]
    pub const fn text(mut self, text: bool) -> Self {
        self.types.set_text(text);
        self
    }

    /// Trims all texts
    ///
    /// This includes removal of text parts that contain only whitespaces, which
    /// is very useful to remove new lines for example:
    ///
    /// # Examples
    ///
    /// ```
    /// use html_filter::*;
    ///
    /// let html = Html::parse(
    ///     "
    /// <!doctype html>
    /// <ul>
    ///     <li>First</li>
    ///     <li>Second></li>
    /// </ul>
    /// ",
    /// )
    /// .unwrap();
    ///
    /// // With trim
    /// let Html::Tag { tag, child, .. } = html.to_filtered(&Filter::new().tag_name("ul").trim())
    /// else {
    ///     panic!()
    /// };
    /// assert_eq!(tag.as_name(), "ul");
    /// let Html::Vec(vec) = *child else { panic!() };
    /// assert!(matches!(vec[0], Html::Tag { .. })); // first li
    /// assert!(matches!(vec[1], Html::Tag { .. })); // second li
    /// assert_eq!(vec.len(), 2);
    ///
    /// // Without trim
    /// let Html::Tag { tag, child, .. } = html.to_filtered(&Filter::new().tag_name("ul")) else {
    ///     panic!()
    /// };
    /// assert_eq!(tag.as_name(), "ul");
    /// let Html::Vec(vec) = *child else { panic!() };
    /// assert_eq!(vec[0], Html::Text("\n    ".to_string()));
    /// assert!(matches!(vec[1], Html::Tag { .. })); // first li
    /// assert_eq!(vec[2], Html::Text("\n    ".to_string()));
    /// assert!(matches!(vec[3], Html::Tag { .. })); // second li
    /// assert_eq!(vec[4], Html::Text("\n".to_string()));
    /// assert_eq!(vec.len(), 5);
    /// ```
    ///
    /// See also [`Self::collapse`]
    #[must_use]
    pub const fn trim(mut self) -> Self {
        self.types.trim();
        self
    }
}

/// Public API for [`Filter`] on tags and attributes
impl Filter {
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
    #[must_use]
    pub fn attribute_name<N: Into<String>>(mut self, name: N) -> Self {
        self.attrs.push(name.into(), AttributeMatch::NoValue, true);
        self
    }

    /// Specifies the value of an attribute in the wanted tags.
    ///
    /// This matches only tag attributes that have the correct value for the
    /// given name. To match only one value inside that values (e.g. class
    /// names), cf. [`Filter::attribute_value_contains`].
    ///
    /// See [`Filter`] for usage information.
    #[must_use]
    pub fn attribute_value<N: Into<String>, V: Into<String>>(mut self, name: N, value: V) -> Self {
        self.attrs.push(name.into(), AttributeMatch::Is(value.into()), true);
        self
    }

    /// Specifies a possible value of an attribute in the wanted tags.
    ///
    /// This matches only tag attributes that have the given value as part of
    /// the space-separated values inside the attribute value (cf. example
    /// below). To match exact value, see [`Filter::attribute_value`].
    ///
    ///
    /// # Examples
    ///
    /// ```
    /// use html_filter::*;
    ///
    /// let html = Html::parse(r#"<div class="some_class other_class" />"#).unwrap();
    /// let filter = Filter::new().attribute_value_contains("class", "some_class");
    ///
    /// if let Html::Tag { tag: Tag { name, .. }, .. } = html.filter(&filter) {
    ///     assert_eq!(name, "div");
    /// } else {
    ///     unreachable!();
    /// }
    /// ```
    #[must_use]
    pub fn attribute_value_contains<N: Into<String>, V: Into<String>>(
        mut self,
        name: N,
        value: V,
    ) -> Self {
        self.attrs.push(name.into(), AttributeMatch::Contains(value.into()), true);
        self
    }

    /// Collapses successive text nodes.
    ///
    /// # Examples
    ///
    /// ```
    /// use html_filter::*;
    ///
    /// let html =
    ///     Html::parse("<div>before <!-- comment --> middle <strong>strong</strong> after</div>")
    ///         .unwrap();
    ///
    /// // Without collapse
    /// assert_eq!(
    ///     Html::Vec(
    ///         vec![
    ///             Html::Text("before ".into()),
    ///             Html::Comment(" comment ".into()),
    ///             Html::Text(" middle ".into()),
    ///             Html::Text("strong".into()),
    ///             Html::Text(" after".into())
    ///         ]
    ///         .into()
    ///     ),
    ///     html.to_filtered(&Filter::new().no_tags().text(true))
    /// );
    ///
    /// // With collapse
    /// assert_eq!(
    ///     Html::Vec(
    ///         vec![
    ///             Html::Text("before ".into()),
    ///             Html::Comment(" comment ".into()),
    ///             Html::Text(" middle strong after".into()),
    ///         ]
    ///         .into()
    ///     ),
    ///     html.to_filtered(&Filter::new().no_tags().text(true).collapse())
    /// );
    /// ```
    #[must_use]
    pub const fn collapse(mut self) -> Self {
        self.types.set_collapse();
        self
    }

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
    /// html_filter::Filter::new().attribute_value("href", "second").depth(0);
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
    /// html_filter::Filter::new().attribute_value("href", "second").depth(1);
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
    /// html_filter::Filter::new().attribute_value("href", "second").depth(2);
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
    #[must_use]
    pub const fn depth(mut self, depth: usize) -> Self {
        self.depth = depth;
        self
    }

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
    #[must_use]
    pub fn except_attribute_name<N: Into<String>>(mut self, name: N) -> Self {
        self.attrs.push(name.into(), AttributeMatch::NoValue, false);
        self
    }

    /// Specifies the value of an attribute in the tags that must be dismissed.
    ///
    /// This matches only tag attributes that have the correct value for the
    /// given name. To filter out on a possible value inside the attribute name,
    /// see [`Filter::except_attribute_value_contains`].
    ///
    /// See [`Filter`] for usage information.
    #[must_use]
    pub fn except_attribute_value<N, V>(mut self, name: N, value: V) -> Self
    where
        N: Into<String>,
        V: Into<String>,
    {
        self.attrs.push(name.into(), AttributeMatch::Is(value.into()), false);
        self
    }

    /// Specifies a possible value of an attribute that must be dismissed.
    ///
    /// This matches only tag attributes that have the given value as part of
    /// the space-separated values inside the attribute value (cf. example
    /// below). To match exact value, see [`Filter::except_attribute_value`].
    ///
    ///
    /// # Examples
    ///
    /// ```
    /// use html_filter::*;
    ///
    /// let html = Html::parse(r#"<div class="some_class other_class" />"#).unwrap();
    /// let filter = Filter::new().except_attribute_value_contains("class", "some_class");
    ///
    /// assert_eq!(html.filter(&filter), Html::Empty);
    /// ```
    #[must_use]
    pub fn except_attribute_value_contains<N: Into<String>, V: Into<String>>(
        mut self,
        name: N,
        value: V,
    ) -> Self {
        self.attrs.push(name.into(), AttributeMatch::Contains(value.into()), false);
        self
    }

    /// Specifies the tag name of the wanted tags.
    ///
    /// See [`Filter`] for usage information.
    #[must_use]
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
    /// use html_filter::*;
    ///
    /// const _FILTER: Filter = Filter::new();
    /// ```
    #[must_use]
    pub const fn new() -> Self {
        Self {
            attrs: ValueAssociateHash::new(),
            depth: 0,
            tags: BlackWhiteList::new(),
            types: NodeTypeFilter::new(),
        }
    }

    /// Disable all tags, except those explicitly whitelisted
    ///
    /// # Example
    ///
    /// ```
    /// use html_filter::*;
    /// let html = Html::parse("<!doctype html><div><!-- comment --></div>").unwrap();
    /// assert_eq!(
    ///     html.to_filtered(&Filter::new().no_tags()),
    ///     Html::parse("<!doctype html><!-- comment -->").unwrap()
    /// );
    ///
    /// let html = Html::parse("z<body>a<div>b<p>c</p>d</div>e</body>y").unwrap();
    /// assert_eq!(
    ///     html.to_filtered(&Filter::new().no_tags().tag_name("div").collapse()),
    ///     Html::parse("<div>bd</div>").unwrap()
    /// );
    /// ```
    #[must_use]
    pub const fn no_tags(mut self) -> Self {
        self.tags.set_default(false);
        self
    }

    /// Specifies the tag name of the wanted tags.
    ///
    /// See [`Filter`] for usage information.
    #[must_use]
    #[expect(unused_must_use, reason = "filter does not yet support results")]
    pub fn tag_name<N: Into<String>>(mut self, name: N) -> Self {
        self.tags.push(name.into(), true);
        self
    }
}
