//! Module that defines an [`Html`] tree.

use core::fmt;

use super::tag::Tag;

/// Dom tree structure to represent the parsed html.
///
/// This tree represents the whole parsed HTML. To create an [`Html`] from a
/// string, use the [`Html::parse`] function.
///
/// # Examples
///
/// ```
/// use html_filter::*;
///
/// let _html: Html = Html::parse(
///     r#"<nav>
///     <!-- Navigation menu -->
///     <ul>
///         <li href="first">First link</li>
///         <li href="second">Second link</li>
///         <li href="third">Third link</li>
///     </ul>
/// </nav>"#,
/// )
/// .unwrap();
/// ```
#[non_exhaustive]
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub enum Html {
    /// Comment block
    ///
    /// # Example
    ///
    /// `<!-- some comment -->`
    Comment(String),
    /// Document tag.
    ///
    /// These are tags with exclamation marks
    ///
    /// # Examples
    ///
    /// `<!doctype html>`
    #[non_exhaustive]
    Doctype {
        /// Name of the tag
        ///
        /// # Examples
        ///
        /// In the previous example, the name is `doctype`.
        name: String,
        /// Attribute of the tag
        ///
        /// # Examples
        ///
        /// In the previous example, the attribute is `html`.
        attr: Option<String>,
    },
    /// Empty html tree
    ///
    /// Corresponds to an empty string
    #[default]
    Empty,
    /// Tag
    ///
    /// # Examples
    ///
    /// - `<div id="blob">content</div>`
    /// - `<div attr />`
    /// - `</>`
    #[non_exhaustive]
    Tag {
        /// Opening tag
        ///
        /// Contains the name of the tag and its attributes.
        tag: Tag,
        /// Child of the tag
        ///
        /// Everything between the opening and the closing tag.
        ///
        /// # Note
        ///
        /// This is always empty if the tag is self-closing.
        child: Box<Self>,
    },
    /// Raw text
    ///
    /// Text outside of a tag.
    ///
    /// # Examples
    ///
    /// In `a<strong>b`, `a` and `b` are [`Html::Text`] elements
    Text(String),
    /// List of nodes
    ///
    /// # Examples
    ///
    /// In `a<strong>b`, the node is a vector, with [`Html::Text`] `a`,
    /// [`Html::Tag`] `strong` [`Html::Text`] `b`.
    Vec(Box<[Self]>),
}

impl<T: AsRef<str>> PartialEq<T> for Html {
    fn eq(&self, other: &T) -> bool {
        *self.to_string() == *other.as_ref()
    }
}

impl Html {
    /// Returns the text of the comment, if this node is a comment.
    ///
    /// # Examples
    ///
    /// ```
    /// use html_filter::*;
    ///
    /// assert_eq!(Html::parse("<!-- some comment -->").unwrap().as_comment(), Some(" some comment "));
    /// assert_eq!(Html::parse("<div>a</div>").unwrap().as_comment(), None);
    /// assert_eq!(Html::parse("not <!-- at --> top-level").unwrap().as_comment(), None);
    /// ```
    #[must_use]
    pub const fn as_comment(&self) -> Option<&str> {
        if let Self::Comment(comment) = self { Some(comment.as_str()) } else { None }
    }

    /// Returns the text of the doctype, if this node is a doctype.
    ///
    /// # Examples
    ///
    /// ```
    /// use html_filter::*;
    ///
    /// assert_eq!(
    ///     Html::parse("<!doctype html>").unwrap().as_doctype(),
    ///     Some(("doctype", Some("html")))
    /// );
    /// assert_eq!(Html::parse("<!xml>").unwrap().as_doctype(), Some(("xml", None)));
    /// assert_eq!(Html::parse("<div>a</div>").unwrap().as_doctype(), None);
    /// assert_eq!(Html::parse("<!not at> top-level").unwrap().as_doctype(), None);
    /// ```
    #[must_use]
    pub const fn as_doctype(&self) -> Option<(&str, Option<&str>)> {
        if let Self::Doctype { name, attr: maybe_attr } = self {
            if let Some(attr) = maybe_attr {
                Some((name.as_str(), Some(attr.as_str())))
            } else {
                Some((name.as_str(), None))
            }
        } else {
            None
        }
    }

    /// Returns the tag, if this node is a tag.
    ///
    /// # Examples
    ///
    /// ```
    /// use html_filter::*;
    ///
    /// let div = Html::parse(r#"<div href="/">a</div>"#).unwrap();
    /// assert_eq!(div.as_tag().unwrap().0.as_name(), "div");
    /// assert_eq!(div.as_tag().unwrap().0.find_attr_value("href"), Some(&"/".to_owned()));
    ///
    /// assert_eq!(Html::parse("<div>a</div>").unwrap().as_tag().unwrap().1.as_text(), Some("a"));
    /// assert_eq!(Html::parse("<p>a</p><p>b</p>").unwrap().as_tag(), None);
    /// ```
    #[must_use]
    pub const fn as_tag(&self) -> Option<(&Tag, &Self)> {
        if let Self::Tag { tag, child } = self { Some((tag, child)) } else { None }
    }

    /// Returns the text, if this node is a text.
    ///
    /// # Examples
    ///
    /// ```
    /// use html_filter::*;
    ///
    /// assert_eq!(Html::parse("text").unwrap().as_text(), Some("text"));
    /// assert_eq!(Html::parse("<div>a</div>").unwrap().as_text(), None);
    /// assert_eq!(Html::parse("<div>a</div>").unwrap().as_tag().unwrap().1.as_text(), Some("a"));
    /// assert_eq!(Html::parse("<p>a</p><p>b</p>").unwrap().as_text(), None);
    /// ```
    #[must_use]
    pub const fn as_text(&self) -> Option<&str> {
        if let Self::Text(text) = self { Some(text.as_str()) } else { None }
    }

    /// Returns the vec, if this isn't a node but a list of nodes.
    ///
    /// # Examples
    ///
    /// ```
    /// use html_filter::*;
    ///
    /// assert_eq!(Html::parse("<div>a</div>").unwrap().as_vec(), None);
    /// let html = Html::parse("<p>a</p>text<!-- comment-->").unwrap();
    /// let vec = html.as_vec().unwrap();
    /// assert_eq!(vec.get(0).unwrap().as_tag().unwrap().0.as_name(), "p");
    /// assert_eq!(vec.get(0).unwrap().as_tag().unwrap().1.as_text(), Some("a"));
    /// assert_eq!(vec.get(1).unwrap().as_text(), Some("text"));
    /// assert_eq!(vec.get(2).unwrap().as_comment(), Some(" comment"));
    /// ```
    #[must_use]
    pub const fn as_vec(&self) -> Option<&[Self]> {
        if let Self::Vec(vec) = self { Some(vec) } else { None }
    }

    /// Checks if an [`Html`] tree is empty
    pub(crate) const fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }

    /// Creates an empty [`Html`]
    #[must_use]
    pub const fn new() -> Self {
        Self::Empty
    }

    /// Trims the texts then allocates a text [`Html`] node if it isn't empty.
    pub(crate) fn trim_text(text: &str) -> Self {
        let trimmed = text.trim();
        if trimmed.is_empty() { Self::Empty } else { Self::Text(trimmed.to_owned()) }
    }
}

impl fmt::Display for Html {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => "".fmt(f),
            Self::Tag { tag, child } if tag.as_name() == "br" => write!(f, "<br>{child}"),
            Self::Tag { tag, child } => write!(f, "<{tag}>{child}</{}>", tag.as_name()),
            Self::Doctype { name, attr } => match (name, attr) {
                (name_str, Some(attr_str)) => write!(f, "<!{name_str} {attr_str}>"),
                (name_str, None) if name_str.is_empty() => write!(f, "<!>"),
                (name_str, None) => write!(f, "<!{name_str} >"),
            },
            Self::Text(text) => text.fmt(f),
            Self::Vec(vec) => vec.iter().try_for_each(|html| html.fmt(f)),
            Self::Comment(content) => write!(f, "<!--{content}-->"),
        }
    }
}
