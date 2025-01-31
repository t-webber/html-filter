use core::{fmt, mem::take};

use crate::push_option;

#[derive(Default)]
pub struct Attribute {
    pub name: String,
    pub value: Option<String>,
}

#[derive(Default)]
pub struct Tag {
    pub name: PrefixName,
    pub attrs: Vec<Attribute>,
}

pub enum TagBuilder {
    Document {
        name: Option<String>,
        attr: Option<String>,
    },
    Open {
        tag: Tag,
    },
    OpenClose {
        tag: Tag,
    },
    Close(PrefixName),
}

#[derive(Default, PartialEq, Eq)]
pub enum PrefixName {
    #[default]
    Empty,
    Name(String),
    Prefix(String, String),
}

impl PrefixName {
    pub(super) fn has_prefix(&self) -> bool {
        matches!(self, Self::Prefix(..))
    }
    pub(super) fn into_name(self) -> Option<String> {
        match self {
            Self::Empty => None,
            Self::Name(name) => Some(name),
            Self::Prefix(..) => panic!("please check with has_prefix before"),
        }
    }
    pub(super) fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }

    pub(super) fn push_char(&mut self, ch: char) {
        match self {
            Self::Empty => todo!(),
            Self::Name(name) | Self::Prefix(_, name) => name.push(ch),
        }
    }

    pub(super) fn push_colon(&mut self) -> Result<(), &'static str> {
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

#[expect(clippy::min_ident_chars)]
impl fmt::Display for PrefixName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => "".fmt(f),
            Self::Name(name) => name.fmt(f),
            Self::Prefix(prefix, name) => write!(f, "{prefix}:{name}"),
        }
    }
}

#[derive(Default)]
pub enum Html {
    #[default]
    None,
    Tag {
        tag: Tag,
        full: bool,
        child: Box<Html>,
    },
    Document {
        name: Option<String>,
        attr: Option<String>,
    },
    Text(String),
    //TODO: Comment,
    Vec(Vec<Html>),
}

impl Html {
    fn empty_box() -> Box<Self> {
        Box::new(Self::default())
    }

    fn is_pushable(&self, is_char: bool) -> bool {
        match self {
            Html::None | Html::Vec(_) => true,
            Html::Tag { tag, full, child } => !*full,
            Html::Document { .. } => false,
            Html::Text(_) => is_char,
        }
    }

    fn from_char(ch: char) -> Self {
        Self::Text(ch.to_string())
    }

    pub(super) fn push_char(&mut self, ch: char) {
        match self {
            Self::None => *self = Self::from_char(ch),
            Self::Document { .. } | Self::Tag { full: true, .. } => {
                *self = Self::Vec(vec![take(self), Self::from_char(ch)])
            }
            Self::Tag {
                child, full: false, ..
            } => child.push_char(ch),
            Self::Text(text) => text.push(ch),
            Self::Vec(vec) => {
                if let Some(last) = vec.last_mut() {
                    if last.is_pushable(true) {
                        return last.push_char(ch);
                    }
                }
                vec.push(Self::from_char(ch))
            }
        }
    }

    pub(super) fn push_node(&mut self, node: Self) {
        match self {
            Self::None => *self = node,
            Self::Text(_) | Self::Document { .. } | Self::Tag { full: true, .. } => {
                *self = Self::Vec(vec![take(self), node])
            }
            Self::Tag {
                child, full: false, ..
            } => child.push_node(node),
            Self::Vec(vec) => {
                if let Some(last) = vec.last_mut() {
                    if last.is_pushable(false) {
                        return last.push_node(node);
                    }
                }
                vec.push(node)
            }
        }
    }

    pub(super) fn push_tag(&mut self, tag: Tag, closed: bool) {
        self.push_node(Html::Tag {
            tag,
            full: closed,
            child: Html::empty_box(),
        });
    }

    fn close_tag_aux(&mut self, name: &PrefixName) -> TagClosingStatus {
        if let Self::Tag {
            tag,
            full: full @ false,
            child,
        } = self
        {
            let status = child.close_tag_aux(name);
            if matches!(status, TagClosingStatus::Full) {
                if &tag.name == name {
                    *full = true;
                    TagClosingStatus::Success
                } else {
                    TagClosingStatus::WrongName(take(&mut tag.name))
                }
            } else {
                status
            }
        } else {
            TagClosingStatus::Full
        }
    }
    pub(super) fn close_tag(&mut self, name: &PrefixName) -> Result<(), String> {
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
}

enum TagClosingStatus {
    Success,
    Full,
    WrongName(PrefixName),
}
