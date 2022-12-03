fn main() {
    // let x = '\u{20F0}';
    // println!("  a b c");
    // println!("  a{x} b{x} c{x}");

    // let red = "\x1b[0;31m";
    // let reset = "\x1b[0m";
    // println!("eeeeeee{red}\u{0301}eeeeee{reset}");

    let s = "abc";

    let mut it = s.char_indices();

    dbg!(it.as_str());
    dbg!(it.next());
    dbg!(it.as_str());
    dbg!(it.next());
    dbg!(it.as_str());
    dbg!(it.next());
    dbg!(it.as_str());
    dbg!(it.next());
}
