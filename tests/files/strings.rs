use html_filter::*;

use super::test_maker;

macro_rules! make_tests {
    ($($name:ident: $filter:expr => $expect:expr)*) => {
        $(
            #[test]
            fn $name() {
                let tree = Html::parse(INPUT).unwrap().filter(&$filter);
                test_maker(stringify!($name), $expect, &tree, $filter, true)
            }
        )*
    };
}

/// !
pub const INPUT: &str = "
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


// TODO: by default, if there is a filter, things outside a tag are deleted (comments, doctype,
// etc.), is this expected?
depth_default_doctype: Filter::new().comment(false).depth(3).tag_name("strong") => "
<html>
    A first text
    <p>
        A <strong>first</strong> text
        <img />
    </p>
</html>"


text_depth_doctype: Filter::new().doctype(true).comment(false).depth(2).tag_name("p") => "
<!DOCTYPE html>
<html>
    A first text
    <p>
        A <strong>first</strong> text
        <img />
    </p>
</html>"

no_depth_force_doctype: Filter::new().doctype(true).comment(false).depth(1).tag_name("p") => "
<!DOCTYPE html>
<html>
    A first text
    <p>
        A <strong>first</strong> text
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
