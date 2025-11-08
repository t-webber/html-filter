use std::fs::read_to_string;

use html_filter::*;

use super::test_maker;

macro_rules! test_find {
    ($($name:ident: $filter:expr => $expect:expr)*) => {
        $(
            #[test]
            fn $name() {
                let content = read_to_string("tests/data/index.html").unwrap();
                let tree = Html::parse(&content).unwrap_or_else(|err| panic!("{err}"));
                let found_cloned = (&tree).to_found(&$filter);
                let found = tree.find(&$filter);
                test_maker(stringify!($name), $expect, found_cloned, $filter);
                test_maker(stringify!($name), $expect, found, $filter);
            }
        )*
    };
}

test_find!(

find_section: Filter::new().tag_name("section") =>
r##"
<section>
    <h2>Forms</h2>
    <form action="#" method="post">
        <input type="sub\mit" id="name" name="name" />
        <input type='sub"mit' value="Submit" />
        <!-- prettier-ignore -->
        <button enabled/>
    </form>
</section>"##

find_failure: Filter::new().tag_name("azerty") => ""

);
