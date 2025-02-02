#[macro_export]
macro_rules! safe_expect {
    ($code:expr, $reason:expr) => {
        $code.ok_or(format!("This is not meant to happen. Please raise an issue on https://github.com/t-webber/html-parser. Thank you for signaling this issue! {}", $reason))?
    };
}

#[macro_export]
macro_rules! safe_unreachable {
    ($reason:expr) => {
        Err(format!("This is not meant to happen. Please raise an issue on https://github.com/t-webber/html-parser. Thank you for signaling this issue! {}", $reason))
    };
}
