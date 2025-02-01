use core::{fmt, mem::take};

use super::tag::{PrefixName, Tag, TagClosingStatus, TagFull};

#[non_exhaustive]
#[derive(Debug, Default)]
pub enum Html {
    Document {
        name: Option<String>,
        attr: Option<String>,
    },
    #[default]
    Empty,
    Tag {
        tag: Tag,
        full: TagFull,
        child: Box<Html>,
    },
    Text(String),
    Vec(Vec<Html>),
    //TODO: Comment,
}

impl Html {
    pub(crate) fn close_tag(&mut self, name: &PrefixName) -> Result<(), String> {
        match self.close_tag_aux(name) {
            TagClosingStatus::Success => Ok(()),
            TagClosingStatus::Full => Err(format!(
                "Invalid closing tag: Found closing tag for '{name}' but all tags are already closed."
            )),
            TagClosingStatus::WrongName(expected) => Err(format!(
                "Invalid closing tag: Found closing tag for '{name}' but '{expected}' is still open."
            )),
        }
    }

    pub(crate) fn close_tag_aux(&mut self, name: &PrefixName) -> TagClosingStatus {
        if let Self::Tag {
            tag,
            full: full @ TagFull::Opened,
            child,
        } = self
        {
            let status = child.close_tag_aux(name);
            if matches!(status, TagClosingStatus::Full) {
                if &tag.name == name {
                    *full = TagFull::Closed;
                    TagClosingStatus::Success
                } else {
                    TagClosingStatus::WrongName(take(&mut tag.name))
                }
            } else {
                status
            }
        } else if let Self::Vec(vec) = self {
            vec.last_mut()
                .map_or(TagClosingStatus::Full, |child| child.close_tag_aux(name))
        } else {
            TagClosingStatus::Full
        }
    }

    pub(crate) fn empty_box() -> Box<Self> {
        Box::new(Self::default())
    }

    pub(crate) fn from_char(ch: char) -> Self {
        Self::Text(ch.to_string())
    }

    const fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }

    pub(crate) const fn is_pushable(&self, is_char: bool) -> bool {
        match self {
            Self::Empty | Self::Vec(_) => true,
            Self::Tag { full, .. } => full.is_open(),
            Self::Document { .. } => false,
            Self::Text(_) => is_char,
        }
    }

    pub(crate) fn push_char(&mut self, ch: char) {
        match self {
            Self::Empty => *self = Self::from_char(ch),
            Self::Tag {
                child,
                full: TagFull::Opened,
                ..
            } => child.push_char(ch),
            Self::Document { .. }
            | Self::Tag {
                full: TagFull::Closed | TagFull::Inline,
                ..
            } => *self = Self::Vec(vec![take(self), Self::from_char(ch)]),
            Self::Text(text) => text.push(ch),
            Self::Vec(vec) => {
                if let Some(last) = vec.last_mut() {
                    if last.is_pushable(true) {
                        return last.push_char(ch);
                    }
                }
                vec.push(Self::from_char(ch));
            }
        }
    }

    pub(crate) fn push_node(&mut self, node: Self) {
        match self {
            Self::Empty => *self = node,
            Self::Tag {
                child,
                full: TagFull::Opened,
                ..
            } => child.push_node(node),
            Self::Text(_)
            | Self::Document { .. }
            | Self::Tag {
                full: TagFull::Closed | TagFull::Inline,
                ..
            } => *self = Self::Vec(vec![take(self), node]),
            Self::Vec(vec) => {
                if let Some(last) = vec.last_mut() {
                    if last.is_pushable(false) {
                        return last.push_node(node);
                    }
                }
                vec.push(node);
            }
        }
    }

    pub(crate) fn push_tag(&mut self, tag: Tag, inline: bool) {
        self.push_node(Self::Tag {
            tag,
            full: if inline {
                TagFull::Inline
            } else {
                TagFull::Opened
            },
            child: Self::empty_box(),
        });
    }
}

#[expect(clippy::min_ident_chars, reason = "keep trait naming")]
impl fmt::Display for Html {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => "".fmt(f)?,
            Self::Tag { tag, full, child } => match full {
                TagFull::Closed => {
                    write!(f, "<{tag}>{child}</{}>", tag.name)
                }
                TagFull::Opened => {
                    write!(f, "<{tag}>{child}")
                }
                TagFull::Inline => {
                    debug_assert!(child.is_empty(), "child can't be pushed if inline");
                    write!(f, "<{tag} />")
                }
            }?,
            Self::Document { name, attr } => match (name, attr) {
                (None, None) => "<!>".fmt(f),
                (None, Some(value)) | (Some(value), None) => write!(f, "<!{value}>"),
                (Some(name_str), Some(attr_str)) => write!(f, "<!{name_str} {attr_str}>"),
            }?,
            Self::Text(text) => text.fmt(f)?,
            Self::Vec(vec) => {
                for html in vec {
                    write!(f, "{html}")?;
                }
            }
        }
        Ok(())
    }
}
