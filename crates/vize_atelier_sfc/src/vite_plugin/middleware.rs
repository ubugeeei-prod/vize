use vize_carton::String;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ViteDevMiddlewareRewrite {
    pub cleaned_url: String,
    pub fs_path: String,
}

pub fn normalize_css_module_filename(filename: &str) -> String {
    let after_nul = filename
        .rfind('\0')
        .map_or(filename, |nul_idx| &filename[nul_idx + 1..]);
    let without_suffix = strip_style_virtual_suffix(after_nul);
    let path = without_suffix
        .find('?')
        .map_or(without_suffix, |query_idx| &without_suffix[..query_idx]);
    String::from(path)
}

pub fn normalize_dev_middleware_url(req_url: &str) -> Option<ViteDevMiddlewareRewrite> {
    if !req_url.contains("__x00__") {
        return None;
    }

    let (url_path, query_suffix) = split_url_query(req_url);
    let cleaned_path = normalize_fs_prefix(remove_encoded_nul(url_path).as_str());
    if !cleaned_path.starts_with("/@fs/") {
        return None;
    }

    let fs_path = &cleaned_path[4..];
    if !fs_path.starts_with('/') || fs_path.ends_with(".vue.ts") {
        return None;
    }

    let mut cleaned_url = String::with_capacity(cleaned_path.len() + query_suffix.len());
    cleaned_url.push_str(cleaned_path.as_str());
    cleaned_url.push_str(query_suffix);
    (cleaned_url.as_str() != req_url).then_some(ViteDevMiddlewareRewrite {
        cleaned_url,
        fs_path: String::from(fs_path),
    })
}

fn strip_style_virtual_suffix(path: &str) -> &str {
    let without_lang = strip_word_extension(path);
    without_lang.strip_suffix(".module").unwrap_or(without_lang)
}

fn strip_word_extension(path: &str) -> &str {
    let bytes = path.as_bytes();
    let mut cursor = bytes.len();
    while cursor > 0 && is_word_byte(bytes[cursor - 1]) {
        cursor -= 1;
    }
    if cursor == bytes.len() || cursor == 0 || bytes[cursor - 1] != b'.' {
        return path;
    }
    &path[..cursor - 1]
}

fn is_word_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

fn split_url_query(url: &str) -> (&str, &str) {
    url.find('?').map_or((url, ""), |query_idx| {
        (&url[..query_idx], &url[query_idx..])
    })
}

fn remove_encoded_nul(path: &str) -> String {
    let marker = "__x00__";
    let mut output = String::with_capacity(path.len());
    let mut remaining = path;
    while let Some(idx) = remaining.find(marker) {
        output.push_str(&remaining[..idx]);
        remaining = &remaining[idx + marker.len()..];
    }
    output.push_str(remaining);
    output
}

fn normalize_fs_prefix(path: &str) -> String {
    if let Some(rest) = path.strip_prefix("/@id//") {
        let mut output = String::with_capacity(rest.len() + "/@fs/".len());
        output.push_str("/@fs/");
        output.push_str(rest);
        output
    } else {
        String::from(path)
    }
}
