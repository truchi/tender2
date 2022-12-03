use tender::style::Style;

fn main() {
    dbg!(std::mem::size_of::<Style>());
    dbg!(std::mem::size_of::<[Style; 2]>());
    dbg!(std::mem::size_of::<Option<Style>>());
}
