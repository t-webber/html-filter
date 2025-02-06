//! Module to define the tag data structure.

use core::fmt;
use core::mem::take;

use crate::safe_unreachable;

/// Name and optionally a value for an attribute of a tag.
///
/// Attributes provide information about a tag. They can consist in a simple name, or also have a value, after an `=` sign. The values are always surrounded either by single or double quotes.
#[derive(Default, Debug)]
pub struct Attribute {
    /// Whether double or single quotes were used to define the value
    ///
    /// Equals `true` if the attribute value was delimited by double quotes, and false otherwise.
    pub double_quote: bool,
    /// Name of the attribute
    ///
    /// # Examples
    ///
    /// In `<div id="blob" />`, the name of the first attribute is `id`.
    pub name: String,
    /// Value of the attribute
    ///
    /// # Examples
    ///
    /// - In `<div id="blob" />`, the value of the first attribute is `"blob"`.
    /// - In `<div enabled />`, the first attribute doesn't have any value.
    pub value: Option<String>,
}

/// [`Tag`] name with optionally a prefix.
///
/// The prefix of a tag name is the part before the colon.
///
/// # Examples
///
/// - In `<a:b id="blob"/>`, the prefix is `a` and the name is `b`.
/// - In `<a id="blob"/>`, the name is `a` and there is no prefix.
#[non_exhaustive]
#[derive(Default, PartialEq, Eq, Debug)]
pub enum PrefixName {
    /// No name in tag
    ///
    /// This means the tag is a fragment (`<>` or `</>`)
    #[default]
    ///
    Empty,
    /// Name of the fragment
    ///
    /// No prefix here, i.e., no colon found.
    Name(String),
    /// Prefix and name of the fragment
    Prefix(String, String),
}

impl PrefixName {
    /// Checks if name contains also a prefix.
    ///
    /// # Examples
    ///
    /// - Returns `true` for the representation of `<pref:tag />`
    /// - Returns `false` for the representation of `<div />`
    pub(crate) const fn has_prefix(&self) -> bool {
        matches!(self, Self::Prefix(..))
    }

    /// Converts the [`PrefixName`] into its prefix and its name.
    pub(crate) fn into_name(self) -> Option<String> {
        match self {
            Self::Empty => None,
            Self::Name(name) => Some(name),
            Self::Prefix(..) => safe_unreachable!("has_prefix called to check"),
        }
    }

    /// Checks if a [`PrefixName`] is empty
    ///
    /// # Note
    ///
    /// A [`PrefixName`] is empty iff it is an [`PrefixName::Empty`] variant, as the others are initialised with a char.
    pub(crate) const fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }

    /// Pushes a character into a [`PrefixName`]
    pub(crate) fn push_char(&mut self, ch: char) {
        match self {
            Self::Empty => *self = Self::Name(ch.to_string()),
            Self::Name(name) | Self::Prefix(_, name) => name.push(ch),
        }
    }

    /// Pushes a colon into a [`PrefixName`]
    ///
    /// This informs us that there was a prefix.
    ///
    /// # Errors
    ///
    /// Returns an error if there is already a prefix, i.e., if a colon as already been found.
    pub(crate) fn push_colon(&mut self) -> Result<(), &'static str> {
        *self = match self {
            Self::Empty => Self::Prefix(String::new(), String::new()),
            Self::Name(name) => Self::Prefix(take(name), String::new()),
            Self::Prefix(..) => {
                return Err(
                    "Found 2 ':' in name! Only alphabetic characters are allowed in tag names.",
                );
            }
        };
        Ok(())
    }
}

#[expect(clippy::min_ident_chars, reason = "keep trait naming")]
impl fmt::Display for PrefixName {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => "".fmt(f),
            Self::Name(name) => name.fmt(f),
            Self::Prefix(prefix, name) => write!(f, "{prefix}:{name}"),
        }
    }
}

/// Tag structure, with its name and attributes
#[derive(Default, Debug)]
pub struct Tag {
    /// Attributes of the tag. See [`Attribute`].
    pub attrs: Vec<Attribute>,
    /// Name of the tag. See [`PrefixName`].
    pub name: PrefixName,
}

#[expect(clippy::min_ident_chars, reason = "keep trait naming")]
impl fmt::Display for Tag {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)?;
        for attr in &self.attrs {
            write!(f, " {}", attr.name)?;
            if let Some(value) = &attr.value {
                let del = if attr.double_quote { '"' } else { '\'' };
                write!(f, "={del}{value}{del}")?;
            }
        }

        Ok(())
    }
}

/// Builder returns by the parser when run on a tag.
#[non_exhaustive]
pub enum TagBuilder {
    /// Closing tag
    ///
    /// # Examples
    ///
    /// `</>` and `</div>`
    Close(PrefixName),
    /// Document tag
    ///
    /// # Examples
    ///
    /// `<!doctype html>`
    Document {
        /// Name of the document tag.
        ///
        /// # Examples
        ///
        /// From the example above, the name is `doctype`.
        name: Option<String>,
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

/// Response type of the attempt to closing a tag.
#[non_exhaustive]
pub enum TagClosingStatus {
    /// No opened tag were found: all were already closed.
    Full,
    /// Tag successfully closed.
    Success,
    /// The last opened tag has the wrong name.
    WrongName(PrefixName),
}

/// Closing type of the tag.
#[derive(Debug)]
#[non_exhaustive]
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
    /// This happens when the tag is not self closing, and the corresponding closing tag has not yet been found.
    ///
    /// # Examples
    ///
    /// This happens when a <div> was read, but </div> was not yet read.
    pub(super) const fn is_open(&self) -> bool {
        matches!(self, Self::Opened)
    }
}
