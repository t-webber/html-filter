//! Module that transforms a [`String`] into an [`Html`] tree.

mod tag;
use core::str::Chars;

use tag::parse_tag;

use crate::prelude::Html;
use crate::types::html_builder::HtmlBuilder;
use crate::types::tag::TagBuilder;

/// Parses an HTML string into a Dom tree.
///
/// # Errors
///
/// This function returns an error when the input HTML's syntax is invalid.
///
/// # Examples
///
/// ```
/// use html_filter::prelude::*;
///
/// let html: &str = r#"
/// <!DOCTYPE html>
/// <html lang="en">
///     <head>
///         <title>Html sample</title>
///     </head>
///     <body>
///         <p>This is an html sample.</p>
///     </body>
/// </html>
/// "#;
/// let tree: Html = parse_html(html).expect("Invalid HTML");
/// assert!(format!("{tree}") == html);
/// ```
#[inline]
pub fn parse_html(html: &str) -> Result<Html, String> {
    let mut tree = HtmlBuilder::default();
    let mut chars = html.chars();
    match parse_html_aux(&mut chars, &mut tree) {
        Ok(()) => Ok(tree.into_html()),
        Err(err) => Err(err),
        //         Err(err) => Err(format!(
        //             "
        // -----------------------------------------
        // An error occurred while parsing the html.
        // -----------------------------------------
        // {tree:#?}
        // -----------------------------------------
        // {tree}
        // -----------------------------------------
        // {err}
        // -----------------------------------------
        // "
        // )),
    }
}

/// Wrapper for the [`parse_html`] function.
fn parse_html_aux(chars: &mut Chars<'_>, tree: &mut HtmlBuilder) -> Result<(), String> {
    let mut dash_count: u32 = 0;
    let mut style = false;
    let mut script = false;
    let mut comment = false;
    while let Some(ch) = chars.next() {
        if !comment && (style || script) {
            if ch == '<' {
                if let Ok(TagBuilder::Close(name)) = parse_tag(chars) {
                    if style && name == "style" {
                        style = false;
                        tree.close_tag(&name)?;
                        continue;
                    }
                    if script && name == "script" {
                        script = false;
                        tree.close_tag(&name)?;
                        continue;
                    }
                }
            }
            tree.push_char(ch);
        } else if ch == '-' {
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
            comment = false;
            dash_count = 0;
        } else {
            for _ in 0..dash_count {
                tree.push_char('-');
            }
            dash_count = 0;
            if comment {
                tree.push_char(ch);
            } else if ch == '<' {
                match parse_tag(chars)? {
                    TagBuilder::Doctype { name, attr } =>
                        tree.push_node(HtmlBuilder::Doctype { name, attr }),
                    TagBuilder::Open(tag) => {
                        if tag.as_name() == "style" {
                            style = true;
                        } else if tag.as_name() == "script" {
                            script = true;
                        }
                        tree.push_tag(tag, false);
                    }
                    TagBuilder::OpenClose(tag) => tree.push_tag(tag, true),
                    TagBuilder::Close(name) => tree.close_tag(&name)?,
                    TagBuilder::OpenComment => {
                        tree.push_comment();
                        comment = true;
                    }
                }
            } else {
                tree.push_char(ch);
            }
        }
    }
    Ok(())
}
