//! Module to parse an opening tag.
//!
//! This module is used when a <d is found in a html string. It can also mean an opening comment.

use crate::{safe_expect, types::tag::PrefixName};
use core::mem::take;
use core::str::Chars;

use crate::types::tag::{Attribute, Tag, TagBuilder};

/// State that informs on position of the '/' closing character.
///
/// This is relatively to the name of the tag.
enum Close {
    /// The '/' was found after the name.
    ///
    /// # Examples
    ///
    /// `<div/>` or `<div id="blob" />`
    After,
    /// The '/' was found before the name.
    ///
    /// # Examples
    ///
    /// `</>`
    Before,
    /// No `/` was found yet.
    ///
    /// Sometimes, it is never found, like in `<div>`.
    None,
}

/// State of the parsing for the tag.
///
/// The elements of this enum are ordered in chronological order, from reading the first character of the name, to reading the last closing character of a value of an attribute.
#[derive(Default, PartialEq, Eq, Debug)]
#[expect(clippy::arbitrary_source_item_ordering, reason = "chronological order")]
enum TagParsingState {
    /// Parser currently reading the name of the tag.
    ///
    /// Waiting for character to continue the name, the end of the tag or space to read attributes.
    #[default]
    Name,
    /// Parser finished the name and/or the previous attribute.
    ///
    /// Waiting for another attribute name or the end of the tag.
    AttributeNone,
    /// Parser currently reading the name of an attribute.
    ///
    /// Waiting for character to continue the name, the end of the tag or a `=` sign to assign a value to this attribute.
    AttributeName(PrefixName),
    /// Parser read the `=` sign after an attribute name.
    ///
    /// Waiting for a `'` or `"` to assign a value to the last attribute.
    AttributeEq,
    /// Parser currently reading the value of an attribute.
    ///
    /// The attribute was started by a single quote `'`.
    AttributeSingle,
    /// Parser currently reading the value of an attribute.
    ///
    /// The attribute was started by a double quote `"`.
    AttributeDouble,
}

/// Function to format the errors for an invalid character in a given context.
fn invalid_err<T>(ch: char, ctx: &str) -> Result<T, String> {
    Err(format!("Invalid character '{ch}' in {ctx}."))
}

/// Parses an opening tag, or an opening comment.
///
/// # Returns
///
/// A [`TagBuilder`] that indicates the type of the tag/comment that was found.
pub fn parse_tag(chars: &mut Chars<'_>) -> Result<TagBuilder, String> {
    let mut tag = Tag::default();
    let mut state = TagParsingState::default();
    let mut close = Close::None;
    let mut bang = false;
    let mut dash = false;

    while let Some(ch) = chars.next() {
        match (&mut state, ch) {
            (TagParsingState::Name, '-') if dash => return Ok(TagBuilder::OpenComment),
            (TagParsingState::Name, '-') if bang => dash = true,
            _ if dash => return invalid_err('-', "doctype"),
            // closing
            (TagParsingState::Name | TagParsingState::AttributeNone, '>') => {
                return return_tag(bang, close, tag);
            }
            (TagParsingState::AttributeName(attr), '>') => {
                tag.attrs.push(Attribute::from(take(attr)));
                return return_tag(bang, close, tag);
            }
            (TagParsingState::Name, '/') if tag.name.is_empty() => close = Close::Before,
            (TagParsingState::Name | TagParsingState::AttributeNone, '/') => close = Close::After,
            (TagParsingState::AttributeName(attr), '/') => {
                tag.attrs.push(Attribute::from(take(attr)));
                close = Close::After;
            }
            // name
            (TagParsingState::Name, '!') => {
                if tag.name.is_empty() {
                    bang = true;
                } else {
                    return invalid_err(ch, "tag name");
                }
            }
            (TagParsingState::Name, ':') => return invalid_err(ch, "tag name"),
            (TagParsingState::Name, _) if ch.is_whitespace() => {
                state = TagParsingState::AttributeNone;
            }
            (TagParsingState::Name, _) => tag.name.push(ch),
            // attribute none: none in progress
            (TagParsingState::AttributeNone, _) if ch.is_whitespace() => (),
            (TagParsingState::AttributeNone, _) => {
                state = TagParsingState::AttributeName(PrefixName::Name(ch.to_string()));
            }
            // attribute name
            (TagParsingState::AttributeName(attr), '=') => {
                tag.attrs.push(Attribute::from(take(attr)));
                state = TagParsingState::AttributeEq;
            }
            (TagParsingState::AttributeName(attr), _) if ch.is_whitespace() => {
                tag.attrs.push(Attribute::from(take(attr)));
                state = TagParsingState::AttributeNone;
            }
            (TagParsingState::AttributeName(attr), ':') => attr.push_colon()?,
            (TagParsingState::AttributeName(attr), _) => attr.push_char(ch),
            // attribute after `=`
            (TagParsingState::AttributeEq, '"') => {
                state = TagParsingState::AttributeDouble;
                safe_expect!(
                    tag.attrs.last_mut(),
                    "Not AttributeNone so last exists at double quote creation."
                )
                .add_value(true);
            }
            (TagParsingState::AttributeEq, '\'') => {
                state = TagParsingState::AttributeSingle;
                safe_expect!(
                    tag.attrs.last_mut(),
                    "Not AttributeNone so last exists at single quote creation."
                )
                .add_value(false);
            }
            (TagParsingState::AttributeEq, _) => {
                return Err(format!(
                    "Invalid character '{ch}': expected '\'' or '\"' after '=' sign."
                ));
            }
            // attribute value
            (TagParsingState::AttributeSingle, '\'') | (TagParsingState::AttributeDouble, '\"') => {
                state = TagParsingState::AttributeNone;
            }
            (TagParsingState::AttributeSingle | TagParsingState::AttributeDouble, _) => {
                safe_expect!(tag.attrs.last_mut(), "Not AttributeNone so last exists")
                    .push_value(ch);
            }
        }
    }
    Err("EOF: Missing closing '>'.".to_owned())
}

/// Builds a [`TagBuilder`] with the parsing information from [`parse_tag`].
fn return_tag(document: bool, close: Close, mut tag: Tag) -> Result<TagBuilder, String> {
    Ok(match (document, close) {
        (true, Close::After) => return invalid_err('/', "doctype"),
        (true, Close::Before) => return invalid_err('!', "closing tag"),
        (true, Close::None) => {
            if tag.attrs.len() >= 2 {
                return Err("Doctype expected at most one attribute.".to_owned());
            }
            let attr = if let Some(attr) = tag.attrs.pop() {
                match attr {
                    Attribute::NameNoValue(PrefixName::Name(name)) => Some(name),
                    Attribute::NameNoValue(PrefixName::Prefix(..)) => {
                        return invalid_err(':', "doctype attribute");
                    }
                    Attribute::NameValue { .. } => {
                        return Err("Doctype attribute must not have a value.".to_owned());
                    }
                }
            } else {
                None
            };
            TagBuilder::Document {
                name: tag.name,
                attr,
            }
        }
        (false, Close::None) => TagBuilder::Open(tag),
        (false, Close::Before) => {
            if !tag.attrs.is_empty() {
                return Err("Closing tags don't support attributes.".to_owned());
            }
            TagBuilder::Close(tag.name)
        }
        (false, Close::After) => TagBuilder::OpenClose(tag),
    })
}
