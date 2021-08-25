/// Build a b-tree map using a nice syntax like in other
/// languages.
#[macro_export]
macro_rules! map {
    // handle the case w/ a trailing comma by removing the comma
    ($($key:expr => $val:expr),*,) => (
        $crate::map!($($key => $val),*)
    );
    // construct a btree map
    ($($key:expr => $val:expr),*) => ({
        #[allow(unused_mut)]
        let mut b_map = ::std::collections::BTreeMap::new();
        $(b_map.insert($key, $val);)*
        b_map
    });
}
