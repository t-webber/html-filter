# Html Parser

![Clippy](https://github.com/t-webber/html-filter/actions/workflows/clippy.yml/badge.svg?branch=main)
![Build](https://github.com/t-webber/html-filter/actions/workflows/build.yml/badge.svg?branch=main)
![Tests](https://github.com/t-webber/html-filter/actions/workflows/tests.yml/badge.svg?branch=main)
![Docs](https://github.com/t-webber/html-filter/actions/workflows/docs.yml/badge.svg?branch=main)
![Fmt](https://github.com/t-webber/html-filter/actions/workflows/fmt.yml/badge.svg?branch=main)
![Coverage](https://github.com/t-webber/html-filter/actions/workflows/coverage.yml/badge.svg?branch=main)

[![github](https://img.shields.io/badge/GitHub-t--webber/html--parser-blue?logo=GitHub)](https://github.com/t-webber/html-parser)
[![license](https://img.shields.io/badge/Licence-MIT-darkgreen)](https://github.com/t-webber/html-parser?tab=MIT-1-ov-file)
[![coverage](https://img.shields.io/badge/Coverage-90%25-purple)](https://github.com/t-webber/html-parser/actions/workflows/nightly.yml)
[![rust-edition](https://img.shields.io/badge/Rust--edition-2024-darkred?logo=Rust)](https://doc.rust-lang.org/stable/edition-guide/rust-2024/)

This is a rust library that parses html source files and allows you to search in and filter this Html with a specific set of rules.

> Do not use this parser to check the syntax of your HTML code. Many HTML files are parsed without any errors by this parser, as the sole objective is to get a parsed version. Only breaking syntax errors raises errors.
>
> Obviously, all valid HTML files work fine.

This is a simple lightweight html parser, that converts an html file (in the `&str` format) to a tree representing the html tags and text.

## Getting started

You can install it with

```shell
cargo add html_filter
```

then us it like this:

```rust
use html_filter::prelude::*;

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
let tree: Html = Html::parse(html).expect("Invalid HTML");

// Now you can use it!
// Beware, this doesn't always work as you can have ways to write the same HTML.
assert!(format!("{tree}") == html);
```

## Find & filter

You can also use the `find` and `filter` methods to manage this html. To do this, you need to create your filtering options with the `Filter` type.

### Filter

```rust
use html_filter::prelude::*;

let html: &str = r##"
  <section>
    <h1>Welcome to My Random Page</h1>
    <nav>
      <ul>
        <li><a href="/home">Home</a></li>
        <li><a href="/about">About</a></li>
        <li><a href="/services">Services</a></li>
        <li><a href="/contact">Contact</a></li>
      </ul>
    </nav>
  </section>
"##;

// Create your filter
let filter = Filter::new().tag_name("li");

// Parse your html
let filtered_tree: Html = Html::parse(html).expect("Invalid HTML").filter(&filter);

// Check the result: filtered_tree contains the 4 lis from the above html string
if let Html::Vec(links) = filtered_tree {
    assert!(links.len() == 4)
} else {
    unreachable!()
}
```

### Find

The finder returns the first element that respects the filter:

```rust
use html_filter::prelude::*;

let html: &str = r##"
  <section>
    <h1>Welcome to My Random Page</h1>
    <nav>
      <ul>
        <li><a href="/home">Home</a></li>
        <li><a href="/about">About</a></li>
        <li><a href="/services">Services</a></li>
        <li><a href="/contact">Contact</a></li>
      </ul>
    </nav>
  </section>
"##;

// Create your filter
let filter = Filter::new().tag_name("a");

// Parse your html
let link: Html = Html::parse(html).expect("Invalid HTML").find(&filter);

// Check the result: link contains `<a href="/home">Home</a>`
if let Html::Tag { tag, child, .. } = link {
    if let Html::Text(text) = *child {
        assert!(tag.as_name() == "a" && text == "Home");
    } else {
        unreachable!()
    }
} else {
    unreachable!()
}
```

## License

Licensed under either of

-   [Apache License, Version 2.0](LICENSE-APACHE)
-   [MIT license](LICENSE-MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
