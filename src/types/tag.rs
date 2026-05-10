//! Module to define the tag data structure.

use core::fmt;
use core::hash::Hash;

/// Name and optionally a value for an attribute of a tag.
///
/// Attributes provide information about a tag. They can consist in a simple
/// name, or also have a value, after an `=` sign. The values are always
/// surrounded either by single or double quotes.
#[allow(
    clippy::allow_attributes,
    clippy::derived_hash_with_manual_eq,
    reason = "hash on enum doesn't depend of variant data"
)]
#[non_exhaustive]
#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub enum Attribute {
    /// Name of the attribute, when it doesn't have a value
    ///
    /// # Examples
    ///
    /// In `<button />`, the name of the attribute is `button`.
    NameNoValue(String),
    /// Name of the attribute
    ///
    /// # Examples
    ///
    /// `<div id="blob"/>`
    NameValue {
        /// Whether double or single quotes were used to define the value
        ///
        /// Equals `true` if the attribute value was delimited by double quotes,
        /// and false otherwise.
        double_quote: bool,
        /// Name of the attribute
        ///
        /// # Examples
        ///
        /// In `<div id="blob" />`, the name of the first attribute is `id`.
        ///
        /// # Note
        ///
        /// Attribute names can have prefixes, like in `<a xlink:href="link"/>`
        name: String,
        /// Value of the attribute
        ///
        /// # Examples
        ///
        /// - In `<div id="blob" />`, the value of the first attribute is
        ///   `"blob"`.
        value: String,
    },
}

impl Attribute {
    /// Returns the name of an attribute
    #[must_use]
    pub const fn as_name(&self) -> &String {
        match self {
            Self::NameNoValue(name) | Self::NameValue { name, .. } => name,
        }
    }

    /// Returns the value of an attribute
    #[must_use]
    pub const fn as_value(&self) -> Option<&String> {
        match self {
            Self::NameNoValue(_) => None,
            Self::NameValue { value, .. } => Some(value),
        }
    }

    /// Returns the value of an attribute
    fn into_value(self) -> Option<String> {
        match self {
            Self::NameNoValue(_) => None,
            Self::NameValue { value, .. } => Some(value),
        }
    }
}

impl From<String> for Attribute {
    fn from(name: String) -> Self {
        Self::NameNoValue(name)
    }
}

impl fmt::Display for Attribute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NameNoValue(prefix_name) => write!(f, " {prefix_name}"),
            Self::NameValue { double_quote, name, value } => write!(f, " {name}").and_then(|()| {
                let del = if *double_quote { '"' } else { '\'' };
                write!(f, "={del}{value}{del}")
            }),
        }
    }
}

/// Tag structure, with its name and attributes
///
/// # Examples
///
/// ```
/// use html_filter::*;
///
/// let Ok(Html::Tag { tag, .. }) = Html::parse("<a enabled href='https://crates.io'>") else {
///     unreachable!()
/// };
///
/// assert_eq!(tag.as_name(), "a");
/// assert_eq!(tag.find_attr_value("enabled"), None);
/// assert_eq!(tag.find_attr_value("href").unwrap(), "https://crates.io");
///
/// let value: String = tag.into_attr_value("href").unwrap();
/// assert_eq!(&value, "https://crates.io");
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tag {
    /// Attributes of the tag. See [`Attribute`].
    pub attrs: Box<[Attribute]>,
    /// Name of the tag.
    ///
    /// # Examples
    ///
    /// - `<div id="blob">` as name `div`
    /// - `<>` as an empty name
    pub name: String,
}

impl Tag {
    /// Returns the attributes of the tag
    ///
    /// # Examples
    ///
    /// ```
    /// use html_filter::*;
    ///
    /// let html = Html::parse("<div id='blob' enabled />").unwrap();
    /// let attrs = html.as_tag().unwrap().0.as_attrs();
    ///
    /// assert_eq!(attrs[0].as_name(), "id");
    /// assert_eq!(attrs[0].as_value().unwrap(), "blob");
    ///
    /// assert_eq!(attrs[1].as_name(), "enabled");
    /// assert_eq!(attrs[1].as_value(), None);
    /// ```
    #[must_use]
    pub const fn as_attrs(&self) -> &[Attribute] {
        &self.attrs
    }

    /// Returns the name of the tag
    ///
    /// # Examples
    ///
    /// ```
    /// use html_filter::*;
    ///
    /// let html = Html::parse("<div />").unwrap();
    /// assert_eq!(html.as_tag().unwrap().0.as_name(), "div");
    /// ```
    #[must_use]
    pub const fn as_name(&self) -> &str {
        self.name.as_str()
    }

    /// Finds the value of the attribute of the given name
    ///
    /// # Returns
    ///
    /// - `Some(value)` if `name = value` is present in the [`Tag`]
    /// - `None` if the attribute doesn't exist, or if it doesn't have a value
    ///
    /// # Examples
    ///
    /// ```
    /// use html_filter::*;
    ///
    /// let html = Html::parse(r#"<a id="std doc" enabled xlink:href="https://std.rs"/>"#).unwrap();
    /// let (tag, _) = html.as_tag().unwrap();
    /// assert_eq!(tag.find_attr_value("enabled"), None);
    /// assert_eq!(tag.find_attr_value("xlink:href").unwrap(), "https://std.rs");
    /// ```
    #[must_use]
    pub fn find_attr_value<T: AsRef<str>>(&self, name: T) -> Option<&String> {
        self.attrs
            .iter()
            .find(|attr| attr.as_name() == name.as_ref())
            .and_then(|attr| attr.as_value())
    }

    /// Finds the value of the attribute of the given name
    ///
    /// # Returns
    ///
    /// - `Some(value)` if `name = value` is present in the [`Tag`]
    /// - `None` if the attribute doesn't exist, or if it doesn't have a value
    ///
    /// # Examples
    ///
    /// ```
    /// use html_filter::*;
    ///
    /// let html = Html::parse(r#"<a enabled/>"#).unwrap();
    ///
    /// if let Html::Tag { tag, .. } = html {
    ///     assert_eq!(tag.into_attr_value("enabled"), None);
    /// } else {
    ///     unreachable!()
    /// }
    /// ```
    ///
    /// ```
    /// use html_filter::*;
    /// let html = Html::parse(r#"<a id="std doc" href="https://std.rs"/>"#).unwrap();
    ///
    /// if let Html::Tag { tag, .. } = html {
    ///     assert_eq!(tag.into_attr_value("href").unwrap(), "https://std.rs");
    /// } else {
    ///     unreachable!()
    /// }
    /// ```
    #[must_use]
    pub fn into_attr_value<T: AsRef<str>>(self, name: T) -> Option<String> {
        self.attrs.into_iter().find(|attr| attr.as_name() == name.as_ref())?.into_value()
    }
}

impl From<(String, Box<[Attribute]>)> for Tag {
    fn from((name, attrs): (String, Box<[Attribute]>)) -> Self {
        Self { attrs, name }
    }
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.name)?;
        self.attrs.iter().try_for_each(|attr| attr.fmt(f))
    }
}

/// Builder returns by the parser when run on a tag.
pub enum TagBuilder {
    /// Closing tag
    ///
    /// # Examples
    ///
    /// `</,>` and `</div>`
    Close(String),
    /// Document tag
    ///
    /// # Examples
    ///
    /// `<!doctype html>`
    Doctype {
        /// Name of the document tag.
        ///
        /// # Examples
        ///
        /// From the example above, the name is `doctype`.
        name: String,
        /// Attribute of the document tag.
        ///
        /// # Examples
        ///
        /// From the example above, the name is `html`.
        attr: Option<String>,
    },
    /// Opening tag
    ///
    /// Doesn't a `/` at the end of the tag declaration.
    ///
    /// # Examples
    ///
    /// `<div>` and `<>` and `<div id="blob" enabled>`
    Open(Tag),
    /// Self-closing tag.
    ///
    /// Contains a `/` at the end of the tag declaration.
    ///
    /// # Examples
    ///
    /// `<p />` and `<div id="blob" enabled />`
    OpenClose(Tag),
    /// Opening block comment
    ///
    /// # Examples
    ///
    /// `<!--`
    OpenComment,
}

/// Closing type of the tag.
#[derive(Debug)]
pub enum TagType {
    /// Closed tag
    ///
    /// This means the closing part of the tag was found.
    ///
    /// # Examples
    ///
    /// `</div>` was read after `<div>`
    Closed,
    /// Opened tag
    ///
    /// This means the closing part of the tag was not yet found.
    ///
    /// # Examples
    ///
    /// `<div>` was read, but not the associated `</div>` yet.
    Opened,
    /// Self-closing tag
    ///
    /// This means tag closes itself, with a '/' character.
    ///
    /// # Examples
    ///
    /// `<div id="blob" />` and `</>`
    SelfClosing,
}

impl TagType {
    /// Checks if tag is still open.
    ///
    /// This happens when the tag is not self closing, and the corresponding
    /// closing tag has not yet been found.
    ///
    /// # Examples
    ///
    /// This happens when a <div> was read, but </div> was not yet read.
    pub(super) const fn is_open(&self) -> bool {
        matches!(self, Self::Opened)
    }
}
