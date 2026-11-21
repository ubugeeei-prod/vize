use vize_l0::String;

pub(crate) fn repeated(n: usize) -> String {
    let mut source = String::from("<script setup>const value=1;</script><template><div>");
    for _ in 0..n {
        source.push_str("<p :title='value'>{{value}}</p>");
    }
    source.push_str("</div></template>");
    source
}
