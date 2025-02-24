use html_parser::prelude::*;

#[test]
fn manual() {
    let html = r##"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8"/>
    <meta name="viewport" content="width=device-width, initial-scale=1.0"/>
    <title>Document</title>
</head>
<body>
</body>
</html>
    "##;
    let tree = parse_html(html).unwrap();
    if let Html::Vec(vec) = &tree {
        for elt in vec {
            if let Html::Tag { tag, child, .. } = elt {
                if tag.as_name() == "html" {
                    if let Html::Vec(vec) = &**child {
                        for elt in vec {
                            if let Html::Tag { tag, child, .. } = elt {
                                if tag.as_name() == "head" {
                                    if let Html::Vec(vec) = &**child {
                                        for elt in vec {
                                            if let Html::Tag { tag, child, .. } = elt {
                                                if tag.as_name() == "title" {
                                                    if let Html::Text(text) = &**child {
                                                        assert!(text == "Document");
                                                        return;
                                                    } else {
                                                        panic!(
                                                            "invalid child of title tag: {child:?}"
                                                        )
                                                    }
                                                }
                                                // media
                                            }
                                        }
                                        panic!("none with name title");
                                    }
                                    panic!("son of head not vec");
                                }
                                panic!("first tag is head");
                            }
                            // reading some text
                        }
                        panic!("none with name head");
                    }
                    panic!("son of html not vec");
                }
                panic!("first tag is html")
            }
            // reading some text & doctype
        }
        panic!("none with name html");
    } else {
        panic!("expected vec");
    }
}
