# Html Parser

This is a simple lightweight html parser, that converts an html file (in the `str` format) to a tree representing the html tags and text.

## Getting started

You can install it with

```shell
cargo add html_parser
```

then us it like this:

```rust
use html_parser::parse::parse_html;
use html_parser::types::{html::*, tag::*};

let html: &str = r#"
<!DOCTYPE html>
<html lang="en">
    <head>
        <title>Html sample</title>
    </head>
    <body>
        <p>This is an html sample.</p>
    </body>
</html>
"#;

// Parse your html
let tree: Html = parse_html(&html).expect("Invalid html.");

// Now you can use it!
assert!(format!("{tree}") == html);
```

## Known issues

Are no yet supported:

-   `style` and `script` tags
-   escaped characters (e.g. `&nbsp;`, `&amp;` or `&copy`.)
-   the output of `fmt` on `Html` is the default formatting of prettier (applicable for spaces inside tags)
