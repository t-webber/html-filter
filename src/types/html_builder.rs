//! Module that defines a builder for the [`Html`] tree.

use core::fmt;
use core::mem::take;

use super::html::Html;
use super::tag::{Tag, TagType};
use crate::errors::{safe_expect, safe_unreachable};

/// Wrapper for bool to manage visibility
#[derive(Debug)]
pub struct CommentFull(bool);

/// Dom tree structure to represent the parsed html.
///
/// This is a builder for [`Html`]. Refer to its documentation for more
/// information.
#[non_exhaustive]
#[derive(Debug, Default)]
pub enum HtmlBuilder {
    /// Comment block
    ///
    /// # Example
    ///
    /// `<!-- some comment -->`
    #[non_exhaustive]
    Comment {
        /// Content of the comment
        ///
        /// # Examples
        ///
        /// In the previous example, the content is `some content`.
        content: String,
        /// Fullness of the comment
        ///
        /// `full` is `true` iff the closing `-->` was found for this comment.
        ///
        /// # Examples
        ///
        /// In the previous example, the content is `some content`.
        full: CommentFull,
    },
    /// Document tag.
    ///
    /// These are tags with exclamation marks
    ///
    /// # Examples
    ///
    /// `<!doctype HtmlBuilder>`
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
        /// In the previous example, the attribute is `HtmlBuilder`.
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
        /// Type of the tag
        ///
        /// The type is the information on the closing style: self-closing
        /// (`<div/>`), opened (`<div>`) or closed (`<div></div>`).
        full: TagType,
        /// Child of the tag
        ///
        /// Everything between the opening and the closing tag.
        ///
        /// # Note
        ///
        /// This is always empty if the tag is self-closing.
        child: Box<HtmlBuilder>,
    },
    /// Raw text
    ///
    /// Text outside of a tag.
    ///
    /// # Examples
    ///
    /// In `a<strong>b`, `a` and `b` are [`HtmlBuilder::Text`] elements
    Text(String),
    /// List of nodes
    ///
    /// # Examples
    ///
    /// In `a<strong>b`, the node is a vector, with [`HtmlBuilder::Text`] `a`,
    /// [`HtmlBuilder::Tag`] `strong` [`HtmlBuilder::Text`] `b`.
    Vec(Vec<HtmlBuilder>),
}

impl HtmlBuilder {
    /// Pushes a block comment into the [`HtmlBuilder`] tree
    pub fn close_comment(&mut self) -> bool {
        match self {
            Self::Comment { full, .. } =>
                if full.0 {
                    false
                } else {
                    full.0 = true;
                    true
                },
            Self::Text(_) | Self::Empty | Self::Doctype { .. } => false,
            Self::Tag { full, child, .. } => full.is_open() && child.close_comment(),
            Self::Vec(vec) =>
                safe_expect!(vec.last_mut(), "Html vec built with one.").close_comment(),
        }
    }

    /// Method to find to close that last opened tag.
    ///
    /// This method finds the opened tag the closest to the leaves.
    pub fn close_tag(&mut self, name: &str) -> Result<(), String> {
        if self.close_tag_aux(name) {
            Ok(())
        } else {
            Err(format!(
                "Invalid closing tag: Found closing tag for '{name}' but it isn't open."
            ))
        }
    }

    /// Wrapper for [`Self::close_tag`].
    ///
    /// # Returns
    ///
    /// `true` iff the tag was successfully closed.
    pub fn close_tag_aux(&mut self, name: &str) -> bool {
        if let Self::Tag { tag, full: full @ TagType::Opened, child } = self {
            child.close_tag_aux(name)
                || (tag.as_name() == name && {
                    *full = TagType::Closed;
                    true
                })
        } else if let Self::Vec(vec) = self {
            vec.last_mut()
                .is_some_and(|child| child.close_tag_aux(name))
        } else {
            false
        }
    }

    /// Boxes an empty tree.
    pub fn empty_box() -> Box<Self> {
        Box::new(Self::default())
    }

    /// Creates a tree for a character.
    pub fn from_char(ch: char) -> Self {
        Self::Text(ch.to_string())
    }

    /// Exports an [`HtmlBuilder`] into an [`Html`]
    pub fn into_html(self) -> Html {
        match self {
            Self::Comment { content, .. } => Html::Comment(content),
            Self::Doctype { name, attr } => Html::Doctype { name, attr },
            Self::Empty => Html::Empty,
            Self::Tag { tag, child, .. } => Html::Tag { tag, child: Box::new(child.into_html()) },
            Self::Text(text) => Html::Text(text),
            Self::Vec(vec) => Html::Vec(vec.into_iter().map(Self::into_html).collect()),
        }
    }

    /// Checks if an [`HtmlBuilder`] tree is pushable.
    ///
    /// This is to check if a new node needs to be created for the next data.
    ///
    /// This method is different if the input is a char or not.
    pub fn is_pushable(&self, is_char: bool) -> bool {
        match self {
            Self::Empty | Self::Vec(_) => safe_unreachable("Vec or Empty can't be in vec"),
            Self::Tag { full, .. } => full.is_open(),
            Self::Doctype { .. } => false,
            Self::Text(_) => is_char,
            Self::Comment { full, .. } => !full.0,
        }
    }

    /// Pushes one character into an [`HtmlBuilder`] tree.
    pub fn push_char(&mut self, ch: char) {
        match self {
            Self::Empty => *self = Self::from_char(ch),
            Self::Tag { child, full: TagType::Opened, .. } => child.push_char(ch),
            Self::Doctype { .. }
            | Self::Tag { full: TagType::Closed | TagType::SelfClosing, .. } =>
                *self = Self::Vec(vec![take(self), Self::from_char(ch)]),
            Self::Text(text) => text.push(ch),
            Self::Vec(vec) => {
                let last = safe_expect!(vec.last_mut(), "Initialised with one element.");
                if last.is_pushable(true) {
                    return last.push_char(ch);
                }
                vec.push(Self::from_char(ch));
            }
            Self::Comment { content, full } => {
                if full.0 {
                    // This means the comment is at the root
                    *self = Self::Vec(vec![take(self), Self::from_char(ch)]);
                } else {
                    content.push(ch);
                }
            }
        }
    }

    /// Pushes a block comment into the [`HtmlBuilder`] tree
    pub fn push_comment(&mut self) {
        self.push_node(Self::Comment { content: String::new(), full: CommentFull(false) });
    }

    /// Pushes an [`HtmlBuilder`] tree into another one.
    ///
    /// This is useful to add comments or push tags for instance.
    pub fn push_node(&mut self, node: Self) {
        match self {
            Self::Empty => *self = node,
            Self::Tag { child, full: TagType::Opened, .. } => child.push_node(node),
            Self::Text(_)
            | Self::Doctype { .. }
            | Self::Tag { full: TagType::Closed | TagType::SelfClosing, .. } =>
                *self = Self::Vec(vec![take(self), node]),
            Self::Vec(vec) => {
                let last = safe_expect!(vec.last_mut(), "Initialised with one element.");
                if last.is_pushable(false) {
                    return last.push_node(node);
                }
                vec.push(node);
            }
            Self::Comment { .. } => safe_unreachable("Pushed parsed not into an unclosed comment."),
        }
    }

    /// Pushes a tag into an [`HtmlBuilder`] tree.
    pub fn push_tag(&mut self, tag: Tag, inline: bool) {
        self.push_node(Self::Tag {
            tag,
            full: if inline {
                TagType::SelfClosing
            } else {
                TagType::Opened
            },
            child: Self::empty_box(),
        });
    }
}

#[expect(clippy::min_ident_chars, reason = "keep trait naming")]
impl fmt::Display for HtmlBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => "".fmt(f),
            Self::Tag { tag, full, child } => match full {
                TagType::Closed => write!(f, "<{tag}>{child}</{}>", tag.as_name()),
                TagType::Opened => write!(f, "<{tag}>{child}"),
                TagType::SelfClosing => write!(f, "<{tag} />"),
            },
            Self::Doctype { name, attr } => match (name, attr) {
                (name_str, Some(attr_str)) => write!(f, "<!{name_str} {attr_str}>"),
                (name_str, None) if name_str.is_empty() => write!(f, "<!>"),
                (name_str, None) => write!(f, "<!{name_str} >"),
            },
            Self::Text(text) => text.fmt(f),
            Self::Vec(vec) => vec.iter().try_for_each(|html| html.fmt(f)),
            Self::Comment { content, full } => {
                f.write_str("<!--")?;
                f.write_str(content)?;
                if full.0 { f.write_str("-->") } else { Ok(()) }
            }
        }
    }
}
