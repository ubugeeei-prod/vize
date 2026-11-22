use vize_l0::String;

pub(crate) fn repeated(n: usize) -> String {
    let mut source = String::from("<script setup>const value=1;</script><template><div>");
    for _ in 0..n {
        source.push_str("<p :title='value'>{{value}}</p>");
    }
    source.push_str("</div></template>");
    source
}

#[cfg(feature = "setup-component-imports")]
pub(crate) fn component_imports(size: usize, kebab: bool) -> std::string::String {
    let mut source = std::string::String::from("<script setup>");
    for index in 0..size {
        source.push_str(&format!(
            "import CardEntry{index} from './Card{index}.vue';"
        ));
    }
    source.push_str("</script><template>");
    for index in 0..size {
        if kebab {
            source.push_str(&format!("<card-entry-{index}/>"));
        } else {
            source.push_str(&format!("<CardEntry{index}/>"));
        }
    }
    source.push_str("</template>");
    source
}
