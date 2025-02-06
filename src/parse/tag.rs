//! Module to parse an opening tag.
//!
//! This module is used when a <d is found in a html string. It can also mean an opening comment.

use crate::safe_expect;
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
#[derive(Default, PartialEq, Eq)]
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
    AttributeName,
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

/// Function to format the errors for an invalid alphanumeric character in a given context.
fn invalid_err_alpha<T>(ch: char, ctx: &str) -> Result<T, String> {
    Err(format!(
        "Invalid character '{ch}' in {ctx}. Only alphanumeric characters are allowed."
    ))
}

/// Parses an opening tag, or an opening comment.
///
/// # Returns
///
/// A [`TagBuilder`] that indicates the type of the tag/comment that was found.
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
                safe_expect!(tag.attrs.last_mut(), "Not AttributeNone so last exists")
                    .name
                    .push(ch);
            }
            (TagParsingState::AttributeName, '=') => state = TagParsingState::AttributeEq,
            (TagParsingState::AttributeNone | TagParsingState::AttributeName, _) => {
                return invalid_err_alpha(ch, "tag attribute names");
            }
            // attribute after `=`
            (TagParsingState::AttributeEq, '"') => {
                state = TagParsingState::AttributeDouble;
                safe_expect!(tag.attrs.last_mut(), "Not AttributeNone so last exists").value =
                    Some(String::new());
            }
            (TagParsingState::AttributeEq, '\'') => {
                state = TagParsingState::AttributeSingle;
                safe_expect!(tag.attrs.last_mut(), "Not AttributeNone so last exists").value =
                    Some(String::new());
            }
            (TagParsingState::AttributeEq, _) => {
                return Err(format!(
                    "Invalid character {ch}: expected '\'' or '\"' after '=' sign."
                ));
            }
            // attribute value
            (TagParsingState::AttributeSingle | TagParsingState::AttributeDouble, _) if escaped => {
                safe_expect!(
                    safe_expect!(tag.attrs.last_mut(), "Not AttributeNone so last exists")
                        .value
                        .as_mut(),
                    "Value created when state changed"
                )
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
                safe_expect!(
                    safe_expect!(tag.attrs.last_mut(), "Not AttributeNone so last exists")
                        .value
                        .as_mut(),
                    "Value created when state changed"
                )
                .push(ch);
            }
        }
    }
    Err("EOF: Missing closing '>'".to_owned())
}

/// Builds a [`TagBuilder`] with the parsing information from [`parse_tag`].
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
                name: tag.name.into_name()?,
                attr: tag.attrs.pop().map(|attr| attr.name),
            }
        }
        (false, Close::None) => TagBuilder::Open(tag),
        (false, Close::Before) => {
            if !tag.attrs.is_empty() {
                return Err("Closing tags doesn't support attributes.".to_owned());
            }
            TagBuilder::Close(tag.name)
        }
        (false, Close::After) => TagBuilder::OpenClose(tag),
    })
}
