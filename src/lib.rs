use std::{mem::take, str::Chars};

use tag::parse_tag;

#[derive(Default)]
struct Attribute {
    name: String,
    value: Option<String>,
}

#[derive(Default)]
struct Attributes(Vec<Attribute>);

#[derive(Default)]
pub struct Tag {
    file: bool,
    prefix: Option<String>,
    name: Option<String>,
    attrs: Attributes,
}

impl Tag {
    fn into_html(self) -> Html {
        Html::Element {
            tag: self,
            full: false,
            child: Box::new(Html::None),
        }
    }
}

#[derive(Default)]
pub enum Html {
    #[default]
    None,
    Element {
        tag: Tag,
        full: bool,
        child: Box<Html>,
    },
    Text(String),
    //TODO: Comment,
    Vec(Vec<Html>),
}

impl Html {
    fn push(&mut self, ch: char) {
        match self {
            Self::None => *self = Self::Text(ch.to_string()),
            Self::Element { child, .. } => child.push(ch),
            Self::Text(text) => text.push(ch),
            Self::Vec(vec) => {
                if let Some(last) = vec.last_mut() {
                    last.push(ch);
                } else {
                    vec.push(Self::Text(ch.to_string()))
                }
            }
        }
    }

    fn push_tag(&mut self, tag: Tag) {
        match self {
            Self::None => *self = tag.into_html(),
            Self::Element { child, .. } => child.push_tag(tag),
            Self::Text(_) => *self = Self::Vec(vec![take(self), tag.into_html()]),
            Self::Vec(vec) => {
                if matches!(vec.last(), None | Some(Self::Text(_))) {
                    vec.push(tag.into_html())
                } else {
                    vec.last_mut().unwrap().push_tag(tag);
                }
            }
        }
    }

    fn fill(&mut self) -> bool {
        match self {
            Self::None => false,
            Self::Element {
                full: full @ false,
                child,
                ..
            } => {
                if !child.fill() {
                    *full = true;
                }
                true
            }
            Self::Element { full: true, .. } => false,
            Self::Text(_) => false,
            Self::Vec(vec) => vec.last_mut().map(|last| last.fill()).unwrap_or(false),
        }
    }
}

pub fn parse_html(html: &str) -> Result<Html, String> {
    let mut chars = html.chars();
    parse_elt(&mut chars)
}

fn parse_elt(chars: &mut Chars<'_>) -> Result<Html, String> {
    let mut tree = Html::default();
    let mut dash_count = 0;
    while let Some(ch) = chars.next() {
        if ch == '-' {
            dash_count += 1;
        } else if ch == '>' && dash_count >= 2 {
            for _ in 0..(dash_count - 2) {
                tree.push('-');
                todo!("close comment")
            }
        } else {
            for _ in 0..dash_count {
                tree.push('-');
            }
            if ch == '<' {
                if let Some(tag) = parse_tag(chars)? {
                    tree.push_tag(tag);
                } else if !tree.fill() {
                    return Err("Mismatched closing tag".to_owned()); //TODO: this is very common, improve error message.
                }
            } else {
                tree.push(ch);
            }
        }
    }
    Ok(tree)
}

mod tag;
