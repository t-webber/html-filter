use html_parser::prelude::*;

use super::test_maker;

const INPUT: &str = "
<!DOCTYPE html>
<!-- comment 1 -->
<html>
    A first text
    <!-- first comment -->
    <p>
        A <strong>first</strong> text
        <!-- second comment -->
        <img />
    </p>
</html>
";

macro_rules! make_tests {
    ($($name:ident: $filter:expr => $expect:expr)*) => {
        $(
            #[test]
            fn $name() {
                let tree = parse_html(INPUT).unwrap().filter(&$filter);
                test_maker(stringify!($name), $expect, tree, $filter)
            }
        )*
    };
}

make_tests!(

all: Filter::new() => "
<!DOCTYPE html>
<!-- comment 1 -->
<html>
    A first text
    <!-- first comment -->
    <p>
        A <strong>first</strong> text
        <!-- second comment -->
        <img />
    </p>
</html>
"

only_tags: Filter::new().all(false) => "
<html>
    <p>
        <strong></strong>
        <img />
    </p>
</html>
"

text: Filter::new().none_except_text() => "
<html>
    A first text
    <p>
        A <strong>first</strong> text
        <img />
    </p>
</html>"

comment: Filter::new().none_except_comment() => "
<!-- comment 1 -->
<html>
    <!-- first comment -->
    <p>
        <strong></strong>
        <!-- second comment -->
        <img />
    </p>
</html>"

doctype: Filter::new().none_except_doctype().no_tags() => "
<!DOCTYPE html>
"

text_comment: Filter::new().all_except_doctype() => "
<!-- comment 1 -->
<html>
    A first text
    <!-- first comment -->
    <p>
        A <strong>first</strong> text
        <!-- second comment -->
        <img />
    </p>
</html>"

text_doctype: Filter::new().all_except_comment() => "
<!DOCTYPE html>
<html>
    A first text
    <p>
        A <strong>first</strong> text
        <img />
    </p>
</html>"

doctype_comment: Filter::new().all_except_text() => "
<!DOCTYPE html>
<!-- comment 1 -->
<html>
    <!-- first comment -->
    <p>
        <strong></strong>
        <!-- second comment -->
        <img />
    </p>
</html>"

);
