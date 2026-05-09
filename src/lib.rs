#![doc = include_str!("../README.md")]

mod errors;
mod filter;
mod parse;
mod types;

pub use crate::filter::types::Filter;
pub use crate::types::html::Html;
pub use crate::types::tag::{Attribute, Tag};

/// A const equivalent of the [`Option::unwrap_or`] method.
const fn unwrap_or(opt: Option<bool>, default: bool) -> bool {
    match opt {
        Some(val) => val,
        None => default,
    }
}
