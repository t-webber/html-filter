use core::str::Chars;

use crate::types::tag::{Attribute, Tag, TagBuilder};

enum Close {
    After,
    Before,
    None,
}

#[expect(clippy::arbitrary_source_item_ordering, reason = "chronological order")]
#[derive(Default, PartialEq, Eq)]
enum TagParsingState {
    #[default]
    Name,
    AttributeNone,
    AttributeName,
    AttributeEq,
    AttributeSingle,
    AttributeDouble,
}

fn invalid_err<T>(ch: char, ctx: &str) -> Result<T, String> {
    Err(format!("Invalid character '{ch}' in {ctx}."))
}

fn invalid_err_alpha<T>(ch: char, ctx: &str) -> Result<T, String> {
    Err(format!(
        "Invalid character '{ch}' in {ctx}. Only alphanumeric characters are allowed."
    ))
}

pub fn parse_tag(chars: &mut Chars<'_>) -> Result<TagBuilder, String> {
    let mut tag = Tag::default();
    let mut state = TagParsingState::default();
    let mut escaped = false;
    let mut close = Close::None;
    let mut document = false;

    while let Some(ch) = chars.next() {
        match (&state, ch) {
            (
                TagParsingState::Name
                | TagParsingState::AttributeNone
                | TagParsingState::AttributeName,
                '>',
            ) => return return_tag(document, close, tag),
            // closing
            (TagParsingState::Name, '/') if tag.name.is_empty() => close = Close::Before,
            (
                TagParsingState::Name
                | TagParsingState::AttributeNone
                | TagParsingState::AttributeName,
                '/',
            ) => close = Close::After,
            // name
            (TagParsingState::Name, 'a'..='z' | 'A'..='Z' | '0'..='9') => tag.name.push_char(ch),
            (TagParsingState::Name, '!') => {
                if tag.name.is_empty() {
                    document = true;
                } else {
                    return invalid_err(ch, "doctype");
                }
            }
            (TagParsingState::Name, ':') => tag.name.push_colon()?,
            // name
            (TagParsingState::Name, _) if ch.is_whitespace() => {
                state = TagParsingState::AttributeNone;
            }
            (TagParsingState::Name, _) => {
                return invalid_err_alpha(ch, "tag names");
            }
            // attribute none: none in progress
            (TagParsingState::AttributeNone, 'a'..='z' | 'A'..='Z' | '0'..='9') => {
                tag.attrs.push(Attribute {
                    name: ch.to_string(),
                    value: None,
                });
                state = TagParsingState::AttributeName;
            }
            (TagParsingState::AttributeNone, _) if ch.is_whitespace() => (),
            // attribute name
            (TagParsingState::AttributeName, 'a'..='z' | 'A'..='Z') => {
                tag.attrs.last_mut().unwrap().name.push(ch);
            }
            (TagParsingState::AttributeName, '=') => state = TagParsingState::AttributeEq,
            (TagParsingState::AttributeNone | TagParsingState::AttributeName, _) => {
                return invalid_err_alpha(ch, "tag attribute names");
            }
            // attribute after `=`
            (TagParsingState::AttributeEq, '"') => {
                state = TagParsingState::AttributeDouble;
                tag.attrs.last_mut().unwrap().value = Some(String::new());
            }
            (TagParsingState::AttributeEq, '\'') => {
                state = TagParsingState::AttributeSingle;
                tag.attrs.last_mut().unwrap().value = Some(String::new());
            }
            (TagParsingState::AttributeEq, _) => {
                return Err(format!(
                    "Invalid character {ch}: expected '\'' or '\"' after '=' sign."
                ));
            }
            // attribute value
            (TagParsingState::AttributeSingle | TagParsingState::AttributeDouble, _) if escaped => {
                tag.attrs
                    .last_mut()
                    .unwrap()
                    .value
                    .as_mut()
                    .unwrap()
                    .push(ch);
                escaped = false;
            }
            (TagParsingState::AttributeSingle | TagParsingState::AttributeDouble, '\\') => {
                escaped = true;
            }

            (TagParsingState::AttributeSingle, '\'') | (TagParsingState::AttributeDouble, '\"') => {
                state = TagParsingState::AttributeNone;
            }
            (TagParsingState::AttributeSingle | TagParsingState::AttributeDouble, _) => {
                tag.attrs
                    .last_mut()
                    .unwrap()
                    .value
                    .as_mut()
                    .unwrap()
                    .push(ch);
            }
        }
    }
    Err("EOF: Missing closing '>'".to_owned())
}

fn return_tag(document: bool, close: Close, mut tag: Tag) -> Result<TagBuilder, String> {
    Ok(match (document, close) {
        (true, Close::After) => return invalid_err('/', "doctype"),
        (true, Close::Before) => return invalid_err('!', "closing tag"),
        (true, Close::None) => {
            if tag.name.has_prefix() {
                return invalid_err(':', "closing tag");
            }
            if tag.attrs.len() >= 2 {
                return Err("Doctype expected at most one attribute.".to_owned());
            }
            if tag.attrs.last().is_some_and(|attr| attr.value.is_some()) {
                return Err("Doctype attribute must not have a value.".to_owned());
            }
            TagBuilder::Document {
                name: tag.name.into_name(),
                attr: tag.attrs.pop().map(|attr| attr.name),
            }
        }
        (false, Close::None) => TagBuilder::Open { tag },
        (false, Close::Before) => {
            if !tag.attrs.is_empty() {
                return Err("Closing tags doesn't support attributes.".to_owned());
            }
            TagBuilder::Close(tag.name)
        }
        (false, Close::After) => TagBuilder::OpenClose { tag },
    })
}
