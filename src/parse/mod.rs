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
    parse_html_aux(&mut chars, &mut tree).map_err(|err| {
        format!(
            "
-----------------------------------------
An error occurred while parsing the html.
-----------------------------------------
{tree:#?}
-----------------------------------------
{err}
-----------------------------------------
"
        )
    })?;
    Ok(tree)
}

/// Wrapper for the [`parse_html`] function.
fn parse_html_aux(chars: &mut Chars<'_>, tree: &mut Html) -> Result<(), String> {
    let mut dash_count: u32 = 0;
    while let Some(ch) = chars.next() {
        if ch == '-' {
            if dash_count >= 2 {
                tree.push_char('-');
            } else {
                #[expect(clippy::arithmetic_side_effects, reason = "checked")]
                {
                    dash_count += 1;
                }
            }
        } else if ch == '>' && dash_count >= 2 {
            #[expect(clippy::arithmetic_side_effects, reason = "checked")]
            for _ in 0..(dash_count - 2) {
                tree.push_char('-');
                todo!("close comment")
            }
        } else {
            for _ in 0..dash_count {
                tree.push_char('-');
            }
            if ch == '<' {
                match parse_tag(chars)? {
                    TagBuilder::Document { name, attr } => {
                        tree.push_node(Html::Document { name, attr });
                    }
                    TagBuilder::Open(tag) => tree.push_tag(tag, false),
                    TagBuilder::OpenClose(tag) => tree.push_tag(tag, true),
                    TagBuilder::Close(name) => tree.close_tag(&name)?,
                }
            } else {
                tree.push_char(ch);
            }
        }
    }
    Ok(())
}
