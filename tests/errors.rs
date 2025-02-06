use html_parser::parse::parse_html;

macro_rules! make_err_test {
    ($($name:ident: $html:expr => $err:expr)*) => {
        $(
            #[test]
            fn $name() {
                let html = $html;
                let expected = $err;
                if let Err(err) = parse_html(html) {
                    assert!(
                        err == expected,
                        "Error mismatch! Expected\n{expected}\nbut found \n{err}\n."
                    )
                } else {
                    panic!("No errors found, but expected:\n{expected}\n.");
                }
            }
        )*
    };
}

make_err_test!(

invalid_char: "<a@b>" => "Invalid character '@' in tag names. Only alphanumeric characters are allowed."

invalid_dash: "<!-a>" => "Invalid character '-' in doctype."

);
