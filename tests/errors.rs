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

bang_closing: "</!doc h>" => "Invalid character '!' in closing tag."
invalid_dash: "<!-audio>" => "Invalid character '-' in doctype."
doctype_col: "<!dx xml:>" => "Invalid character ':' in doctype attribute."
doctype_val: "<!dx h=''>" => "Doctype attribute must not have a value."
close_doctype: "<!doc />" => "Invalid character '/' in doctype."
invalid_bang: "<button!>" => "Invalid character '!' in tag name."
double_colon: "<ab c::d>" => "Found 2 colons ':' in attribute name."
prefix_name: "<image:br>" => "Invalid character ':' in tag name."
invalid_equal: "<p id=a>" => "Invalid character 'a': expected ''' or '\"' after '=' sign."
unclosed_tag: "<textarea" => "EOF: Missing closing '>'."
unopened_tag: "<br></em>" => "Invalid closing tag: Found closing tag for 'em' but it isn't open."
unopened_comment: " --> " => "Tried to close unopened comment."
attr_close: "</a id='c'>" => "Closing tags don't support attributes."
second_close: "<!---->-->" => "Tried to close unopened comment."
doctype_2attr: "<!dx a b>" => "Doctype expected at most one attribute."

);
