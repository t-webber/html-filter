use std::{mem, str::Chars};

use crate::{Attribute, Tag};

#[derive(Default, PartialEq, Eq)]
enum TagParsingState {
    #[default]
    Prefix,
    Name,
    AttributeNone,
    AttributeName,
    AttributeEq,
    AttributeSingle,
    AttributeDouble,
}

pub fn parse_tag(chars: &mut Chars<'_>) -> Result<Option<Tag>, String> {
    let mut tag = Tag::default();
    let mut state = TagParsingState::default();
    let mut escaped = false;

    while let Some(ch) = chars.next() {
        match (&state, ch) {
            (
                TagParsingState::Prefix
                | TagParsingState::Name
                | TagParsingState::AttributeNone
                | TagParsingState::AttributeName,
                '>',
            ) => return Ok(Some(tag)),
            // closing
            (TagParsingState::Prefix, '/') => return Ok(None),
            // prefix
            (TagParsingState::Prefix, 'a'..='z' | 'A'..='Z') => {
                push_option(&mut tag.prefix, ch);
            }
            (TagParsingState::Prefix, '!') => {
                tag.file = true;
                state = TagParsingState::Name
            }
            (TagParsingState::Prefix, ':') => {
                tag.file = true;
                state = TagParsingState::Name
            }
            // name
            (TagParsingState::Name, 'a'..='z' | 'A'..='Z') => {
                push_option(&mut tag.name, ch);
            }
            (TagParsingState::Prefix | TagParsingState::Name, _) if ch.is_whitespace() => {
                if tag.name.is_none() {
                    mem::swap(&mut tag.prefix, &mut tag.name);
                }
                state = TagParsingState::AttributeNone;
            }
            (TagParsingState::Prefix | TagParsingState::Name, _) => {
                return Err(format!(
                    "Invalid character {ch}: only alphabetic characters are allowed in tag names."
                ));
            }
            // attribute none: none in progress
            (TagParsingState::AttributeNone, 'a'..='z' | 'A'..='Z') => {
                tag.attrs.0.push(Attribute {
                    name: ch.to_string(),
                    value: None,
                });
                state = TagParsingState::AttributeName;
            }
            (TagParsingState::AttributeNone, _) if ch.is_whitespace() => (),
            // attribute name
            (TagParsingState::AttributeName, 'a'..='z' | 'A'..='Z') => {
                tag.attrs.0.last_mut().unwrap().name.push(ch);
            }
            (TagParsingState::AttributeName, '=') => state = TagParsingState::AttributeEq,
            (TagParsingState::AttributeNone | TagParsingState::AttributeName, _) => {
                return Err(format!(
                    "Invalid character {ch}: only alphabetic characters are allowed in attribute names."
                ));
            }
            // attribute after `=`
            (TagParsingState::AttributeEq, '"') => {
                state = TagParsingState::AttributeDouble;
                tag.attrs.0.last_mut().unwrap().value = Some(String::new())
            }
            (TagParsingState::AttributeEq, '\'') => {
                state = TagParsingState::AttributeSingle;
                tag.attrs.0.last_mut().unwrap().value = Some(String::new())
            }
            (TagParsingState::AttributeEq, _) => {
                return Err(format!(
                    "Invalid character {ch}: expected '\'' or '\"' after '=' sign."
                ));
            }
            // attribute value
            (TagParsingState::AttributeSingle | TagParsingState::AttributeDouble, _) if escaped => {
                tag.attrs
                    .0
                    .last_mut()
                    .unwrap()
                    .value
                    .as_mut()
                    .unwrap()
                    .push(ch);
                escaped = false;
            }
            (TagParsingState::AttributeSingle | TagParsingState::AttributeDouble, '\\') => {
                escaped = true
            }
            (TagParsingState::AttributeSingle, '\'') | (TagParsingState::AttributeDouble, '\"') => {
                state = TagParsingState::AttributeNone
            }
            (TagParsingState::AttributeSingle | TagParsingState::AttributeDouble, _) => {
                tag.attrs
                    .0
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

fn push_option(opt: &mut Option<String>, ch: char) {
    if let Some(string) = opt {
        string.push(ch)
    } else {
        *opt = Some(ch.to_string())
    }
}
