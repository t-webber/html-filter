#![expect(
    clippy::while_let_on_iterator,
    reason = "better to understand when the iterator is used after the loop brakes"
)]
#![allow(unused, reason = "dev")]

use std::str::Chars;

use tag::parse_tag;
use types::{Html, TagBuilder};

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
                        tree.push_node(Html::Document { name, attr })
                    }
                    TagBuilder::Open { tag } => tree.push_tag(tag, false),
                    TagBuilder::OpenClose { tag } => tree.push_tag(tag, true),
                    TagBuilder::Close(name) => tree.close_tag(&name)?,
                }
            } else {
                tree.push_char(ch);
            }
        }
    }
    Ok(tree)
}

mod tag;
mod types;

fn push_option(opt: &mut Option<String>, ch: char) {
    if let Some(string) = opt {
        string.push(ch)
    } else {
        *opt = Some(ch.to_string())
    }
}
