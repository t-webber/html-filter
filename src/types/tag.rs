use core::fmt;
use core::mem::take;

use crate::safe_unreachable;

#[derive(Default, Debug)]
pub struct Attribute {
    pub name: String,
    pub value: Option<String>,
}

#[non_exhaustive]
#[derive(Default, PartialEq, Eq, Debug)]
pub enum PrefixName {
    #[default]
    Empty,
    Name(String),
    Prefix(String, String),
}

impl PrefixName {
    pub(crate) const fn has_prefix(&self) -> bool {
        matches!(self, Self::Prefix(..))
    }

    pub(crate) fn into_name(self) -> Result<Option<String>, String> {
        match self {
            Self::Empty => Ok(None),
            Self::Name(name) => Ok(Some(name)),
            Self::Prefix(..) => safe_unreachable!("has_prefix called to check"),
        }
    }

    pub(crate) const fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }

    pub(crate) fn push_char(&mut self, ch: char) {
        match self {
            Self::Empty => *self = Self::Name(ch.to_string()),
            Self::Name(name) | Self::Prefix(_, name) => name.push(ch),
        }
    }

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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => "".fmt(f),
            Self::Name(name) => name.fmt(f),
            Self::Prefix(prefix, name) => write!(
                f,
                "{prefix}
            :{name}
            "
            ),
        }
    }
}

#[derive(Default, Debug)]
pub struct Tag {
    pub attrs: Vec<Attribute>,
    pub name: PrefixName,
}

#[expect(clippy::min_ident_chars, reason = "keep trait naming")]
impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}
        ",
            self.name
        )?;
        for attr in &self.attrs {
            write!(
                f,
                " {}
            ",
                attr.name
            )?;
            if let Some(value) = &attr.value {
                write!(
                    f,
                    "=\"{value}
                \""
                )?;
            }
        }

        Ok(())
    }
}

#[non_exhaustive]
pub enum TagBuilder {
    Close(PrefixName),
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
}

#[non_exhaustive]
pub enum TagClosingStatus {
    Full,
    Success,
    WrongName(PrefixName),
}

#[derive(Debug)]
#[non_exhaustive]
pub enum TagFull {
    Closed,
    Inline,
    Opened,
}

impl TagFull {
    pub(super) const fn is_open(&self) -> bool {
        matches!(self, Self::Opened)
    }
}
