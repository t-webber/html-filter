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

invalid_dash: "<!-audio>" => "Invalid character '-' in doctype."
close_doctype: "<!doc />" => "Invalid character '/' in doctype."
bang_closing: "</!doc >" => "Invalid character '!' in closing tag."
prefix_name: "<webkit:br>" => "Invalid character ':' in tag name."
invalid_bang: "<button!>" => "Invalid character '!' in tag name."
invalid_equal: "<p id=a>" => "Invalid character 'a': expected ''' or '\"' after '=' sign."
unclosed_tag: "<textarea" => "EOF: Missing closing '>'."
unopened_tag: "<br></div>" => "Invalid closing tag: Found closing tag for 'div' but it isn't open."

);
