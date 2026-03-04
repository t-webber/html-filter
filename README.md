# html-filter

> Parse HTML into a typed tree, then search and filter it — zero dependencies.

[![github](https://img.shields.io/badge/GitHub-t--webber/html--filter-blue?logo=GitHub)](https://github.com/t-webber/html-filter)
[![license](https://img.shields.io/badge/Licence-MIT%20OR%20Apache--2.0-darkgreen)](https://github.com/t-webber/html-filter?tab=MIT-1-ov-file)
[![coverage](https://img.shields.io/badge/Coverage-95%25-purple)](https://github.com/t-webber/html-filter/actions/workflows/nightly.yml)
[![rust-edition](https://img.shields.io/badge/Rust--edition-2024-darkred?logo=Rust)](https://doc.rust-lang.org/stable/edition-guide/rust-2024/)

![Clippy](https://github.com/t-webber/html-filter/actions/workflows/clippy.yml/badge.svg?branch=main)
![Build](https://github.com/t-webber/html-filter/actions/workflows/build.yml/badge.svg?branch=main)
![Tests](https://github.com/t-webber/html-filter/actions/workflows/tests.yml/badge.svg?branch=main)
![Docs](https://github.com/t-webber/html-filter/actions/workflows/rustdoc.yml/badge.svg?branch=main)
![Fmt](https://github.com/t-webber/html-filter/actions/workflows/rustfmt.yml/badge.svg?branch=main)
![Coverage](https://github.com/t-webber/html-filter/actions/workflows/coverage.yml/badge.svg?branch=main)

---

## What is this?

`html-filter` is a small Rust library for working with HTML. You give it an HTML string; it gives back a typed recursive tree. You can then walk that tree directly, or use the built-in `Filter` API to select, exclude, and extract exactly what you need.

This comes in handy for extracting links, headings, metadata, or any specific elements or text from an HTML document. It is also useful to clean-up some html by removing the scripts and style tags, comments, etc.

It is not intended for validating HTML (the parser is lenient by design as page often contain invalid syntax).

## Installation

```shell
cargo add html_filter
```

## Quick start

```rust
use html_filter::*;

let src = r#"
<!DOCTYPE html>
<html lang="en">
  <head>
    <title>My Page</title>
  </head>
  <body>
    <h1>Hello!</h1>
    <p>Welcome to my page.</p>
  </body>
</html>
"#;

let tree: Html = Html::parse(src).expect("invalid HTML");
```

## Filtering

`Filter` uses a builder pattern: start with `Filter::new()` and chain as many conditions as you need. Call `.filter()` on a parsed tree to get back an `Html` containing every node that matched. Here are a few examples:

### Select by tag name

```rust
use html_filter::*;

let src = r#"
  <nav>
    <a href="/home">Home</a>
    <a href="/about">About</a>
    <a href="/contact">Contact</a>
  </nav>
"#;

let filter = Filter::new().tag_name("a");
let result = Html::parse(src).unwrap().filter(&filter);

// All three <a> tags are collected into an Html::Vec.
if let Html::Vec(links) = result {
    assert_eq!(links.len(), 3);
}
```

### Select by attribute value

```rust
use html_filter::*;

let src = r#"
  <form>
    <input type="text"   name="username" />
    <input type="password" name="pass" />
    <input type="submit" value="Login" />
  </form>
"#;

// Find only the submit button.
let filter = Filter::new().attribute_value("type", "submit");
let result = Html::parse(src).unwrap().find(&filter);

if let Html::Tag { tag, .. } = result {
    assert_eq!(tag.find_attr_value("value").unwrap(), "Login");
}
```

### Select by CSS class (space-separated values)

HTML class attributes can contain several class names separated by spaces.
`attribute_value_contains` matches when the given word appears anywhere in that list.

```rust
use html_filter::*;

let src = r#"
  <ul>
    <li class="item featured">Rust</li>
    <li class="item">Go</li>
    <li class="item featured">Zig</li>
  </ul>
"#;

// Grab only the featured items.
let filter = Filter::new().attribute_value_contains("class", "featured");
let result = Html::parse(src).unwrap().filter(&filter);

if let Html::Vec(items) = result {
    assert_eq!(items.len(), 2);
}
```

### Exclude tags or attributes

Every positive selector has a negative counterpart. They can be mixed freely.

```rust
use html_filter::*;

let src = r#"
  <div>
    <p class="visible">Keep me</p>
    <p class="hidden">Discard me</p>
    <script>alert('also gone')</script>
  </div>
"#;

// Keep everything, but strip <script> tags and elements with class "hidden".
let filter = Filter::new()
    .except_tag_name("script")
    .except_attribute_value_contains("class", "hidden");

let result = Html::parse(src).unwrap().filter(&filter);

// Only the first <p> survives.
if let Html::Tag { tag, .. } = result {
    assert_eq!(tag.as_name(), "p");
}
```

## Finding

`find` is a shorthand for `filter` that returns only the first matching node instead of all of them. Use it when you know there is exactly one element you care about.

```rust
use html_filter::*;

let src = r##"
  <article>
    <h1 id="title">Getting Started</h1>
    <p>First paragraph.</p>
    <p>Second paragraph.</p>
  </article>
"##;

// Get the element whose id is "title".
let filter = Filter::new().attribute_value("id", "title");
let heading = Html::parse(src).unwrap().find(&filter);

if let Html::Tag { tag, child, .. } = heading
    && let Html::Text(text) = *child
{
    assert_eq!(tag.as_name(), "h1");
    assert_eq!(text, "Getting Started");
}
```

## The `depth` option: retrieve context around a match

By default, `filter` returns exactly the nodes that matched. Setting `depth(n)` tells the filter to also keep up to `n` levels of ancestors around each match. This is very useful when you want to keep a tag based on it's content and not on the tag itself.

```rust
use html_filter::*;

let src = r#"<nav><ul>
  <li href="first">First</li>
  <li href="second">Second</li>
  <li href="third">Third</li>
</ul></nav>"#;

// depth(0) — default: return only the matched <li>.
let filter = Filter::new().attribute_value("href", "second");
if let Html::Vec(items) = Html::parse(src).unwrap().filter(&filter) {
    if let Html::Tag { tag, .. } = &items[0] {
        assert_eq!(tag.as_name(), "li");
    }
}

// depth(1): return the <ul> that contains the matched <li>.
let filter = Filter::new().attribute_value("href", "second").depth(1);
if let Html::Tag { tag, .. } = Html::parse(src).unwrap().filter(&filter) {
    assert_eq!(tag.as_name(), "ul");
}

// depth(2): return the <nav>.
let filter = Filter::new().attribute_value("href", "second").depth(2);
if let Html::Tag { tag, .. } = Html::parse(src).unwrap().filter(&filter) {
    assert_eq!(tag.as_name(), "nav");
}
```

---

## Filtering node types

You can strip or keep comments, doctype declarations, and text nodes independently of tag filtering.

```rust
use html_filter::*;

let src = r#"<!DOCTYPE html>
<!-- page header -->
<h1>Title</h1>
"#;

// Keep only tags; remove the doctype and comments.
let filter = Filter::new()
    .tag_name("h1")
    .comment(false)
    .doctype(false);

let result = Html::parse(src).unwrap().filter(&filter);

if let Html::Tag { tag, .. } = result {
    assert_eq!(tag.as_name(), "h1");
}
```

Convenience methods for common cases:

```rust
use html_filter::*;

let src = r#"<!DOCTYPE html><!-- comment --><p>text</p>"#;

// Strip all `<!-- -->` comments.
let r = Html::parse(src).unwrap().filter(&Filter::new().comment(false));
assert_eq!(r.to_string(), "<!DOCTYPE html><p>text</p>");

// Strip all `<!…>` doctype nodes.
let r = Html::parse(src).unwrap().filter(&Filter::new().doctype(false));
assert_eq!(r.to_string(), "<!-- comment --><p>text</p>");

// Strip all bare text nodes.
let r = Html::parse(src).unwrap().filter(&Filter::new().text(false));
assert_eq!(r.to_string(), "<!DOCTYPE html><!-- comment --><p></p>");

// Keep everything except comments.
let r = Html::parse(src).unwrap().filter(&Filter::new().all_except_comment());
assert_eq!(r.to_string(), "<!DOCTYPE html><p>text</p>");

// Keep only text nodes (no comments or doctypes).
let r = Html::parse(src).unwrap().filter(&Filter::new().none_except_text());
assert_eq!(r.to_string(), "<p>text</p>");
```

---

## Inspecting tags and attributes

Once you have an `Html::Tag`, you can interrogate its `Tag` and `Attribute`s directly.

```rust
use html_filter::*;

let src = r#"<a id="crates" href="https://crates.io" enabled>crates.io</a>"#;
let html = Html::parse(src).unwrap();

if let Html::Tag { tag, child, .. } = html {
    // Name of the tag.
    assert_eq!(tag.as_name(), "a");

    // Read an attribute value by name (returns None for value-less attributes).
    assert_eq!(tag.find_attr_value("href").unwrap(), "https://crates.io");
    assert!(tag.find_attr_value("enabled").is_none()); // value-less

    // Inner text.
    if let Html::Text(text) = *child {
        assert_eq!(text, "crates.io");
    }
}
```

`into_attr_value` consumes the tag and returns the value as an owned `String`, useful when you want to move the string out without cloning:

```rust
use html_filter::*;

let src = r#"<meta name="description" content="A great page." />"#;

if let Html::Tag { tag, .. } = Html::parse(src).unwrap() {
    let content: String = tag.into_attr_value("content").unwrap();
    assert_eq!(content, "A great page.");
}
```

## Borrowing vs. consuming

All operations have both a consuming variant (takes `self`) and a borrowing variant (takes `&self`):

```rust
use html_filter::*;

let src = r#"<ul><li>one</li><li>two</li></ul>"#;
let filter = Filter::new().tag_name("li");
let html = Html::parse(src).unwrap();

// Borrowing variants keep the original value.
let filtered = html.to_filtered(&filter);
let first = html.to_found(&filter);

// Consuming variants take ownership.
let filtered = html.clone().filter(&filter);
let first = html.find(&filter);

if let Html::Tag { tag, .. } = first {
    assert_eq!(tag.as_name(), "li");
}
```

## License

Licensed under either of

- [Apache License, Version 2.0](LICENSE-APACHE)
- [MIT license](LICENSE-MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
