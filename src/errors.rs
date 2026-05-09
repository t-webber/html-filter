//! Module that defines macros to deal with developer errors.
//!
//! These errors are those made by coding, i.e., are never mean't to be fired.
//! If they happen, it is asked to the user to raise an issue on the system
//! version control.

#![expect(clippy::arbitrary_source_item_ordering, reason = "macro used")]

/// Macro to add a developer error with a generic failure text.
macro_rules! safe_expect {
    ($code:expr, $reason:expr) => {
        $code.expect(&$crate::errors::unreachable_message($reason))
    };
}

/// Macro to make a developer error with a generic failure text.
macro_rules! safe_unreachable {
    ($reason:expr) => {{
        $crate::errors::safe_unreachable_fn($reason);
    }};
}

/// Function to make a developer error with a generic failure text.
#[expect(clippy::panic, reason = "called when code must fail to avoid undefined behaviour.")]
pub fn safe_unreachable_fn(reason: &str) -> ! {
    panic!("{}", unreachable_message(reason))
}

/// Formats the message to display when something unexpected happens.
pub fn unreachable_message(reason: &str) -> String {
    format!("\nThis is not meant to happen.\nPlease report this problem at https://github.com/t-webber/html-parser/issues/new.\nPlease include the code snippet that created this error and the reason displayed below.\nThank you for signalling this issue!\nWe will try to fix it as soon as possible.\n---------- Reason ----------\n{reason}\n----------------------------\n")
}

pub(super) use safe_expect;
pub(super) use safe_unreachable;
