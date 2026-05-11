# html-filter

---

_Parse HTML into a typed tree, then search for tags, attributes, classes, filter out comments or find the exact data you want with a short builder pattern in no time — zero dependencies, zero overhead._

---

[![github](https://img.shields.io/badge/GitHub-html--filter-blue?logo=GitHub)](https://github.com/t-webber/html-filter)
[![license](https://img.shields.io/badge/Licence-MIT%20OR%20Apache--2.0-darkgreen)](https://github.com/t-webber/html-filter?tab=MIT-1-ov-file)
[![coverage](https://img.shields.io/badge/Coverage-100%25-purple)](https://github.com/t-webber/html-filter/actions/workflows/nightly.yml)
[![rust-edition](https://img.shields.io/badge/Rust--edition-2024-darkred?logo=Rust)](https://doc.rust-lang.org/stable/edition-guide/rust-2024/)

![lint](https://github.com/t-webber/html-filter/actions/workflows/clippy.yml/badge.svg?branch=main)
![build](https://github.com/t-webber/html-filter/actions/workflows/build.yml/badge.svg?branch=main)
![test](https://github.com/t-webber/html-filter/actions/workflows/tests.yml/badge.svg?branch=main)
![doc](https://github.com/t-webber/html-filter/actions/workflows/rustdoc.yml/badge.svg?branch=main)
![format](https://github.com/t-webber/html-filter/actions/workflows/rustfmt.yml/badge.svg?branch=main)
![coverage](https://github.com/t-webber/html-filter/actions/workflows/coverage.yml/badge.svg?branch=main)

---

## Why use this crate ?

- For HTML parsing and filtering
- Lightweight and no dependencies
- Public and accessible HTML tree representation
- Easy interface to filter HTML
- Extract some information from some HTML in just a few lines
- Lenient parsing to not crash on non-valid HTML files
- Contextual Filtering: retrieve ancestors of matched nodes, to keep a node based on child content.

## Installation

```shell
cargo add html_filter
```

You first need to parse the HTML data:

```rust
use html_filter::*;

let html = Html::parse("<div class='hidden'>Secret</div><p>Public</p>").unwrap();
let filter = Filter::new().except_attribute_value("class", "hidden");
assert_eq!(html.filter(&filter), "<p>Public</p>");
```

## Filtering

`Filter` uses a builder pattern: start with `Filter::new()` and chain as many conditions as you need. Call `.filter()` on a parsed tree to get back an `Html` containing every node that matched. You can also call `.find()` to return only the first node that matches the conditions, or `to_filtered`/`to_found` to not consume the `Html` but still only clone only what is necessary. Here are a few examples:

### Select by tag name

```rust
use html_filter::*;

let html = Html::parse(r#"
  <nav>
    <a href="/home">Home</a>
    <a href="/about">About</a>
    <a href="/contact">Contact</a>
  </nav>
"#).unwrap();

let filter = Filter::new().tag_name("a");
let result = html.filter(&filter);

// All three <a> tags are collected into an Html::Vec.
assert_eq!(result,
    r#"<a href="/home">Home</a><a href="/about">About</a><a href="/contact">Contact</a>"#
);

// If you want to access the text, you can unwrap this html:
let link_text = result.as_vec().unwrap().iter().map(|a| {
    let (tag, child) = a.as_tag().unwrap();
    let href = tag.find_attr_value("href").unwrap();
    (href.as_str(), child.as_text().unwrap())
}).collect::<Vec<_>>();

assert_eq!(link_text, vec![("/home", "Home"), ("/about", "About"), ("/contact", "Contact")]);
```

### Select by attribute value

```rust
use html_filter::*;

let html = Html::parse(r#"
  <form>
    <input type="text"   name="username" />
    <input type="password" name="pass" />
    <input type="submit" value="Login" />
  </form>
"#).unwrap();

// Find only the submit button.
let filter = Filter::new().attribute_value("type", "submit");
let result = html.find(&filter);

if let Html::Tag { tag, .. } = result {
    assert_eq!(tag.find_attr_value("value").unwrap(), "Login");
} else {
    unreachable!();
}
```

### Select by CSS class (space-separated values)

```rust
use html_filter::*;

let html = Html::parse(r#"
  <ul>
    <li class="item featured">Rust</li>
    <li class="item">Go</li>
    <li class="item featured">Zig</li>
  </ul>
"#).unwrap();

// Grab only the featured items.
let filter = Filter::new().attribute_value_contains("class", "featured");
let result = html.filter(&filter);
let items = result.as_vec().unwrap();

assert_eq!(items.len(), 2);
assert_eq!(items[0].as_tag().unwrap().1.as_text(), Some("Rust"));
assert_eq!(items[1].as_tag().unwrap().1.as_text(), Some("Zig"));
```

### Exclude tags or attributes, and exclude white spaces

```rust
use html_filter::*;

let html = Html::parse(r#"
  <div>
    <style> .p { background: #222; }</style>
    <p class="visible">Keep me</p>
    <p class="hidden">Discard me</p>
    <script>alert('also gone')</script>
  </div>
"#).unwrap();

let filter = Filter::new()
    .trim() // removes white spaces after removing tags
    .except_tag_name("script")
    .except_tag_name("style")
    .except_attribute_value_contains("class", "hidden");

let result = html.filter(&filter);
assert_eq!(result, r#"<div><p class="visible">Keep me</p></div>"#);

let (div, inner) = result.as_tag().unwrap();
assert_eq!(div.as_name(), "div");

let (p, text) = inner.as_tag().unwrap();
assert_eq!(p.as_name(), "p");
assert_eq!(p.find_attr_value("class").unwrap(), "visible");
assert_eq!(text.as_text().unwrap(), "Keep me");
```

### The `depth` option: retrieve context around a match

By default, `filter` returns exactly the nodes that matched. Setting `depth(n)` tells the filter to also keep up to `n` levels of ancestors around each match. This is very useful when you want to keep a tag based on its content and not on the tag itself.

```rust
use html_filter::*;

let html = Html::parse(r#"<nav><ul>
  <li href="first">First</li>
  <li href="second">Second</li>
  <li href="third">Third</li>
</ul></nav>"#).unwrap();

// depth(0) — default: return only the matched <li>.
let filter = Filter::new().attribute_value("href", "second").depth(0);
if let Html::Vec(items) = html.to_filtered(&filter) {
    if let Html::Tag { tag, .. } = &items[0] {
        assert_eq!(tag.as_name(), "li");
    }
}

// depth(1): return the <ul> that contains the matched <li>.
let filter = Filter::new().attribute_value("href", "second").depth(1);
if let Html::Tag { tag, .. } = html.to_filtered(&filter) {
    assert_eq!(tag.as_name(), "ul");
}

// depth(2): return the <nav>.
let filter = Filter::new().attribute_value("href", "second").depth(2);
if let Html::Tag { tag, .. } = html.filter(&filter) {
    assert_eq!(tag.as_name(), "nav");
}
```

### Filtering node types

You can strip or keep comments, doctype declarations, and text nodes independently of tag filtering.

```rust
use html_filter::*;

let html = Html::parse(r#"<!DOCTYPE html>
<!-- page header -->
<h1>Title</h1>
"#).unwrap();

let filter = Filter::new()
    .tag_name("h1")
    .text(false) // remove text even if in tag
    .doctype(true) // force keep doctype even if not in tag
    ;

assert_eq!(html.filter(&filter), "<!DOCTYPE html><h1></h1>");
```

Convenience methods for common cases:

```rust
use html_filter::*;

let html = Html::parse(r#"<!DOCTYPE html><!-- comment --><p>text</p>"#).unwrap();

// Strip all `<!-- -->` comments.
let r = html.to_filtered(&Filter::new().comment(false));
assert_eq!(r.to_string(), "<!DOCTYPE html><p>text</p>");

// Strip all `<!…>` doctype nodes.
let r = html.to_filtered(&Filter::new().doctype(false));
assert_eq!(r.to_string(), "<!-- comment --><p>text</p>");

// Strip all bare text nodes.
let r = html.to_filtered(&Filter::new().text(false));
assert_eq!(r.to_string(), "<!DOCTYPE html><!-- comment --><p></p>");

// Keep everything except comments.
let r = html.to_filtered(&Filter::new().all_except_comment());
assert_eq!(r.to_string(), "<!DOCTYPE html><p>text</p>");

// Keep only text nodes (no comments or doctypes).
let r = html.to_filtered(&Filter::new().none_except_text());
assert_eq!(r.to_string(), "<p>text</p>");
```

### Text extraction

```rust
use html_filter::*;

let html = Html::parse(r#"
<ul>
  <li>first,</li>
  <li>second,</li>
  <li>third</li>
</ul>
"#).unwrap();

let text = html.filter(&Filter::new().collapse().trim().no_tags().none_except_text());

assert_eq!(text.as_text().unwrap(), "first,second,third");
```

## Inspecting tags and attributes

Once you have an `Html::Tag`, you can interrogate its `Tag` and `Attribute`s directly.

```rust
use html_filter::*;

let html = Html::parse(r#"<a id="crates" href="https://crates.io" enabled>crates.io</a>"#).unwrap();
let (tag, child) = html.as_tag().unwrap();

// Tag name
assert_eq!(tag.as_name(), "a");

// Tag attributes
assert_eq!(tag.find_attr_value("href").unwrap(), "https://crates.io");
assert_eq!(tag.find_attr_value("enabled"), None);

// Tag content
assert_eq!(child.as_text().unwrap(), "crates.io");
```

`into_attr_value` consumes the tag and returns the value as an owned `String`, useful when you want to move the string out without cloning:

```rust
use html_filter::*;

if let Html::Tag { tag, .. } = Html::parse(r#"<meta name="description" content="A great page." />"#).unwrap() {
    let content: String = tag.into_attr_value("content").unwrap();
    assert_eq!(content, "A great page.");
}
```
