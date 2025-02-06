//! Module that defines macros to deal with developer errors.
//!
//! These errors are those made by coding, i.e., are never mean't to be fired. If they happen, it is asked to the user to raise an issue on the system version control.

/// Macro to add a developer error with a generic failure text.
#[macro_export]
macro_rules! safe_expect {
    ($code:expr, $reason:expr) => {
        $code.expect(&format!("
This is not meant to happen. Please raise an issue on https://github.com/t-webber/html-parser. Thank you for signaling this issue!
---------- Reason ----------
{}
----------------------------
", $reason))
    };
}

/// Macro to make a developer error with a generic failure text.
#[inline]
#[coverage(off)]
#[expect(
    clippy::panic,
    reason = "called when code must fail to avoid undefined behaviour."
)]
pub fn safe_unreachable(reason: &str) -> ! {
    panic!("
This is not meant to happen. Please raise an issue on https://github.com/t-webber/html-parser. Thank you for signaling this issue!
---------- Reason ----------
{reason}
----------------------------
"   )
}
