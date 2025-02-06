#![doc = include_str!("../README.md")]
#![warn(
    missing_docs,
    warnings,
    deprecated_safe,
    future_incompatible,
    keyword_idents,
    let_underscore,
    nonstandard_style,
    refining_impl_trait,
    rust_2018_compatibility,
    rust_2018_idioms,
    rust_2021_compatibility,
    rust_2024_compatibility,
    unused,
    clippy::all,
    clippy::pedantic,
    clippy::style,
    clippy::perf,
    clippy::complexity,
    clippy::correctness,
    clippy::restriction,
    clippy::nursery,
    clippy::cargo
)]
#![allow(
    clippy::single_call_fn,
    clippy::implicit_return,
    clippy::mod_module_files,
    clippy::exhaustive_structs,
    clippy::question_mark_used,
    clippy::pattern_type_mismatch,
    clippy::module_name_repetitions,
    clippy::blanket_clippy_restriction_lints,
    reason = "These lint acceptable behaviour."
)]
#![expect(
    clippy::while_let_on_iterator,
    reason = "better to understand when the iterator is used after the loop brakes"
)]
#![expect(clippy::doc_include_without_cfg, reason = "see issue #13918")]

mod errors;
pub mod parse;
pub mod types;
