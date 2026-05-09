//! Module to parse an opening tag.
//!
//! This module is used when a <d is found in a html string. It can also mean an
//! opening comment.

use core::str::Chars;

use super::AUTO_CLOSING_TAGS;
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

impl TagBuilder {
    /// Parses an opening tag, or an opening comment.
    ///
    /// # Returns
    ///
    /// A [`TagBuilder`] that indicates the type of the tag/comment that was
    /// found.
    pub fn parse(chars: &mut Chars<'_>) -> Result<Self, String> {
        let mut state = TagParsingState::default();
        let mut close = Close::None;
        let mut bang = false;
        let mut dash = false;
        let mut tag = String::new();
        let mut attrs = vec![];

        for ch in chars.by_ref() {
            state = match (state, ch) {
                (TagParsingState::Name, '-') if dash => return Ok(Self::OpenComment),
                (old @ TagParsingState::Name, '-') if bang => {
                    dash = true;
                    old
                }
                _ if dash => return invalid_err('-', "doctype"),
                // closing
                (TagParsingState::Name | TagParsingState::AttributeNone, '>') =>
                    return Self::return_tag(bang, close, tag, attrs),
                (TagParsingState::AttributeName(attr), '>') => {
                    attrs.push(Attribute::from(attr));
                    return Self::return_tag(bang, close, tag, attrs);
                }
                (old @ TagParsingState::Name, '/') if tag.is_empty() => {
                    close = Close::Before;
                    old
                }
                (old @ (TagParsingState::Name | TagParsingState::AttributeNone), '/') => {
                    close = Close::After;
                    old
                }
                (TagParsingState::AttributeName(attr), '/') => {
                    attrs.push(Attribute::from(attr));
                    close = Close::After;
                    TagParsingState::AttributeName(String::new())
                }
                // name
                (old @ TagParsingState::Name, '!') =>
                    if tag.is_empty() {
                        bang = true;
                        old
                    } else {
                        return invalid_err(ch, "tag name");
                    },
                (TagParsingState::Name, ':') => return invalid_err(ch, "tag name"),
                (TagParsingState::Name, _) if ch.is_whitespace() => TagParsingState::AttributeNone,
                (old @ TagParsingState::Name, _) => {
                    tag.push(ch);
                    old
                }
                // attribute none: none in progress
                (old @ TagParsingState::AttributeNone, _) if ch.is_whitespace() => old,
                (TagParsingState::AttributeNone, _) =>
                    TagParsingState::AttributeName(ch.to_string()),
                // attribute name
                (TagParsingState::AttributeName(attr), '=') => TagParsingState::AttributeEq(attr),
                (TagParsingState::AttributeName(attr), _) if ch.is_whitespace() => {
                    attrs.push(Attribute::from(attr));
                    TagParsingState::AttributeNone
                }
                (TagParsingState::AttributeName(mut attr), _) => {
                    attr.push(ch);
                    TagParsingState::AttributeName(attr)
                }
                // attribute after `=`
                (TagParsingState::AttributeEq(name), quote @ ('"' | '\'')) =>
                    TagParsingState::AttributeValue {
                        double: quote == '"',
                        name,
                        value: String::new(),
                    },
                (TagParsingState::AttributeEq(_), _) =>
                    return Err(format!(
                        "Invalid character '{ch}': expected '\'' or '\"' after '=' sign."
                    )),
                // attribute value
                (TagParsingState::AttributeValue { double, name, value }, _)
                    if double && ch == '"' || !double && ch == '\'' =>
                {
                    attrs.push(Attribute::NameValue { double_quote: double, name, value });
                    TagParsingState::AttributeNone
                }

                (TagParsingState::AttributeValue { double, name, mut value }, _) => {
                    value.push(ch);
                    TagParsingState::AttributeValue { double, name, value }
                }
            };
        }
        Err("EOF: Missing closing '>'.".to_owned())
    }

    /// Builds a [`TagBuilder`] with the parsing information from
    /// [`TagBuilder::parse`].
    fn return_tag(
        doctype: bool,
        close: Close,
        name: String,
        mut attrs: Vec<Attribute>,
    ) -> Result<Self, String> {
        Ok(match (doctype, close) {
            (true, Close::After) => return invalid_err('/', "doctype"),
            (true, Close::Before) => return invalid_err('!', "closing tag"),
            (true, Close::None) => {
                if attrs.len() >= 2 {
                    return Err("Doctype expected at most one attribute.".to_owned());
                }
                let attr = if let Some(attr) = attrs.pop() {
                    match attr {
                        Attribute::NameNoValue(prefix_name) => Some(prefix_name),
                        Attribute::NameValue { .. } =>
                            return Err("Doctype attribute must not have a value.".to_owned()),
                    }
                } else {
                    None
                };
                Self::Doctype { name, attr }
            }
            (false, Close::None) if AUTO_CLOSING_TAGS.contains(&name.as_str()) =>
                Self::OpenClose(Tag::from((name, attrs.into_boxed_slice()))),
            (false, Close::None) => Self::Open(Tag::from((name, attrs.into_boxed_slice()))),
            (false, Close::Before) => {
                if !attrs.is_empty() {
                    return Err("Closing tags don't support attributes.".to_owned());
                }
                Self::Close(name)
            }
            (false, Close::After) => Self::OpenClose(Tag::from((name, attrs.into_boxed_slice()))),
        })
    }
}

/// State of the parsing for the tag.
///
/// The elements of this enum are ordered in chronological order, from reading
/// the first character of the name, to reading the last closing character of a
/// value of an attribute.
#[derive(Default, PartialEq, Eq, Debug)]
#[expect(clippy::arbitrary_source_item_ordering, reason = "chronological order")]
enum TagParsingState {
    /// Parser currently reading the name of the tag.
    ///
    /// Waiting for character to continue the name, the end of the tag or space
    /// to read attributes.
    #[default]
    Name,
    /// Parser finished the name and/or the previous attribute.
    ///
    /// Waiting for another attribute name or the end of the tag.
    AttributeNone,
    /// Parser currently reading the name of an attribute.
    ///
    /// Waiting for character to continue the name, the end of the tag or a `=`
    /// sign to assign a value to this attribute.
    AttributeName(String),
    /// Parser read the `=` sign after an attribute name.
    ///
    /// Waiting for a `'` or `"` to assign a value to the last attribute.
    AttributeEq(String),
    /// Parser currently reading the value of an attribute.
    AttributeValue {
        /// Whether the value was started with `"` or `'`.
        double: bool,
        /// Name of the attribute, read-only.
        name: String,
        /// Current value, in the process of being built.
        value: String,
    },
}

/// Function to format the errors for an invalid character in a given context.
fn invalid_err<T>(ch: char, ctx: &str) -> Result<T, String> {
    Err(format!("Invalid character '{ch}' in {ctx}."))
}
