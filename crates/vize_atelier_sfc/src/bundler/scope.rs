use std::path::{Component, Path};

use sha2::{Digest, Sha256};
use vize_carton::String;

pub fn generate_bundler_scope_id(
    filename: &str,
    root: Option<&str>,
    is_production: bool,
    source: Option<&str>,
) -> String {
    let input = if let Some(root) = root {
        let relative = relative_scope_path(filename, root);
        if is_production && let Some(source) = source {
            let normalized_source = normalize_newlines(source);
            let mut input = String::with_capacity(relative.len() + normalized_source.len() + 1);
            input.push_str(relative.as_str());
            input.push('\n');
            input.push_str(normalized_source.as_str());
            input
        } else {
            relative
        }
    } else {
        normalize_path(filename)
    };

    sha256_prefix(input.as_str(), 8)
}

fn relative_scope_path(filename: &str, root: &str) -> String {
    let file_components = normal_components(Path::new(filename));
    let root_components = normal_components(Path::new(root));
    let mut common = 0usize;
    while common < file_components.len()
        && common < root_components.len()
        && file_components[common].as_str() == root_components[common].as_str()
    {
        common += 1;
    }

    let parent_count = root_components.len().saturating_sub(common);
    let mut parts: Vec<&str> = Vec::with_capacity(parent_count + file_components.len() - common);
    parts.extend(std::iter::repeat_n("..", parent_count));
    parts.extend(file_components[common..].iter().map(String::as_str));

    let mut start = 0usize;
    while parts.get(start) == Some(&"..") {
        start += 1;
    }
    join_slash(&parts[start..])
}

fn normal_components(path: &Path) -> Vec<String> {
    let mut components = Vec::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => {
                components.push(String::from(part.to_string_lossy().as_ref()))
            }
            Component::ParentDir => components.push(String::from("..")),
            Component::CurDir | Component::RootDir | Component::Prefix(_) => {}
        }
    }
    components
}

fn normalize_path(path: &str) -> String {
    let mut output = String::with_capacity(path.len());
    for char in path.chars() {
        output.push(if char == '\\' { '/' } else { char });
    }
    output
}

fn normalize_newlines(source: &str) -> String {
    if !source.contains("\r\n") {
        return String::from(source);
    }
    let mut output = String::with_capacity(source.len());
    let mut chars = source.chars().peekable();
    while let Some(char) = chars.next() {
        if char == '\r' && chars.peek() == Some(&'\n') {
            continue;
        }
        output.push(char);
    }
    output
}

fn sha256_prefix(input: &str, len: usize) -> String {
    let digest = Sha256::digest(input.as_bytes());
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(len);
    for byte in digest.iter().take(len.div_ceil(2)) {
        output.push(HEX[(byte >> 4) as usize] as char);
        if output.len() == len {
            break;
        }
        output.push(HEX[(byte & 0x0f) as usize] as char);
        if output.len() == len {
            break;
        }
    }
    output
}

fn join_slash(parts: &[&str]) -> String {
    let len = parts.iter().map(|part| part.len()).sum::<usize>() + parts.len().saturating_sub(1);
    let mut output = String::with_capacity(len);
    for (index, part) in parts.iter().enumerate() {
        if index > 0 {
            output.push('/');
        }
        output.push_str(part);
    }
    output
}
