//! Module that transforms a [`String`] into an [`Html`] tree.

mod tag;
use core::str::Chars;

use crate::Html;
use crate::types::html_builder::HtmlBuilder;
use crate::types::tag::TagBuilder;

/// Tags that cannot have a content
///
/// This means that they are always self-closing tags: `<meta>` and `<br>` are
/// closed.
const AUTO_CLOSING_TAGS: [&str; 2] = ["meta", "br"];

impl Html {
    /// Parses an HTML string into a Dom tree.
    ///
    /// # Errors
    ///
    /// This function returns an error when the input HTML's syntax is invalid.
    ///
    /// # Examples
    ///
    /// ```
    /// use html_filter::*;
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
    /// let tree: Html = Html::parse(html).expect("Invalid HTML");
    /// assert!(format!("{tree}") == html);
    /// ```
    pub fn parse(html: &str) -> Result<Self, String> {
        let mut tree = HtmlBuilder::default();
        tree.parse(&mut html.chars()).map(|()| tree.into_html())
    }
}

impl HtmlBuilder {
    /// Wrapper for the [`Html::parse`] method.
    ///
    /// This method transforms a flow of chars into an Html tree.
    fn parse(&mut self, chars: &mut Chars<'_>) -> Result<(), String> {
        let mut dash_count: u32 = 0;
        let mut style = false;
        let mut script = false;
        let mut comment = false;
        while let Some(ch) = chars.next() {
            if !comment && (style || script) {
                if ch == '<'
                    && let Ok(TagBuilder::Close(name)) = TagBuilder::parse(chars)
                {
                    if style && name == "style" {
                        style = false;
                        self.close_tag(&name)?;
                        continue;
                    }
                    if script && name == "script" {
                        script = false;
                        self.close_tag(&name)?;
                        continue;
                    }
                }
                self.push_char(ch);
            } else if ch == '-' {
                #[expect(clippy::arithmetic_side_effects, reason = "checked")]
                if dash_count == 2 {
                    self.push_char('-');
                } else {
                    dash_count += 1;
                }
            } else if ch == '>' && dash_count == 2 {
                if !self.close_comment() {
                    return Err("Tried to close unopened comment.".to_owned());
                }
                comment = false;
                dash_count = 0;
            } else {
                for _ in 0..dash_count {
                    self.push_char('-');
                }
                dash_count = 0;
                if comment {
                    self.push_char(ch);
                } else if ch == '<' {
                    match TagBuilder::parse(chars)? {
                        TagBuilder::Doctype { name, attr } =>
                            self.push_node(Self::Doctype { name, attr }),
                        TagBuilder::Open(tag) => {
                            if tag.as_name() == "style" {
                                style = true;
                            } else if tag.as_name() == "script" {
                                script = true;
                            }
                            self.push_tag(tag, false);
                        }
                        TagBuilder::OpenClose(tag) => self.push_tag(tag, true),
                        TagBuilder::Close(name) => self.close_tag(&name)?,
                        TagBuilder::OpenComment => {
                            self.push_comment();
                            comment = true;
                        }
                    }
                } else {
                    self.push_char(ch);
                }
            }
        }
        Ok(())
    }
}
