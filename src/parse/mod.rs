//! Module that transforms a [`String`] into an [`Html`] tree.

mod tag;
use core::str::Chars;

use tag::parse_tag;

use crate::types::{html::Html, tag::TagBuilder};

/// Parses html into a Dom tree.
///
/// # Errors
///
/// This function returns an error when the html input as an invalid syntax.
#[inline]
pub fn parse_html(html: &str) -> Result<Html, String> {
    let mut tree = Html::default();
    let mut chars = html.chars();
    match parse_html_aux(&mut chars, &mut tree) {
        Ok(()) => Ok(tree),
        #[cfg(not(feature = "debug"))]
        Err(err) => Err(err),
        #[cfg(feature = "debug")]
        Err(err) => Err(format!(
            "
-----------------------------------------
An error occurred while parsing the html.
-----------------------------------------
{tree:#?}
-----------------------------------------
{tree}
-----------------------------------------
{err}
-----------------------------------------
"
        )),
    }
}

/// Wrapper for the [`parse_html`] function.
fn parse_html_aux(chars: &mut Chars<'_>, tree: &mut Html) -> Result<(), String> {
    let mut dash_count: u32 = 0;
    while let Some(ch) = chars.next() {
        if ch == '-' {
            #[expect(clippy::arithmetic_side_effects, reason = "checked")]
            if dash_count == 2 {
                tree.push_char('-');
            } else {
                dash_count += 1;
            }
        } else if ch == '>' && dash_count == 2 {
            if !tree.close_comment() {
                return Err("Tried to close unopened comment.".to_owned());
            }
            dash_count = 0;
        } else {
            for _ in 0..dash_count {
                tree.push_char('-');
            }
            dash_count = 0;
            if tree.is_comment() {
                tree.push_char(ch);
            } else if ch == '<' {
                match parse_tag(chars)? {
                    TagBuilder::Document { name, attr } => {
                        tree.push_node(Html::Document { name, attr });
                    }
                    TagBuilder::Open(tag) => tree.push_tag(tag, false),
                    TagBuilder::OpenClose(tag) => tree.push_tag(tag, true),
                    TagBuilder::Close(name) => tree.close_tag(&name)?,
                    TagBuilder::OpenComment => tree.push_comment(),
                }
            } else {
                tree.push_char(ch);
            }
        }
    }
    Ok(())
}
