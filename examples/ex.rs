use unicode_segmentation::UnicodeSegmentation;

fn main() {
    let s = "\0a̐\0éö̲\r\n";
    let g = UnicodeSegmentation::graphemes(s, true).collect::<Vec<&str>>();
    let c = collect(g);
    dbg!(c);
}

fn collect<I: IntoIterator>(i: I) -> Vec<I::Item> {
    i.into_iter().collect()
}
