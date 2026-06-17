use vize_atelier_core::{
    ParserOptions, TemplateSyntaxMode, parser::parse_with_options_and_template_syntax,
};
use vize_atelier_sfc::{SfcDescriptor, script::resolve_template_used_identifiers};
use vize_carton::{Bump, FxHashSet, String as CompactString};

pub(super) fn collect_art_template_referenced_names(
    descriptor: &SfcDescriptor<'_>,
    template_syntax: TemplateSyntaxMode,
) -> FxHashSet<CompactString> {
    let mut names = FxHashSet::default();

    for block in &descriptor.custom_blocks {
        if block.block_type.as_ref() != "art" {
            continue;
        }
        collect_variant_template_referenced_names(&block.content, template_syntax, &mut names);
    }

    names
}

fn collect_variant_template_referenced_names(
    art_content: &str,
    template_syntax: TemplateSyntaxMode,
    names: &mut FxHashSet<CompactString>,
) {
    let mut cursor = 0;
    while let Some(relative_start) = art_content[cursor..].find("<variant") {
        let start = cursor + relative_start;
        let after_name = start + "<variant".len();
        if !is_variant_tag_boundary(art_content.as_bytes().get(after_name).copied()) {
            cursor = after_name;
            continue;
        }

        let Some(tag_end) = art_content[start..].find('>').map(|offset| start + offset) else {
            break;
        };
        let tag = art_content[start..=tag_end].trim_end();
        if tag.ends_with("/>") {
            cursor = tag_end + 1;
            continue;
        }

        let template_start = tag_end + 1;
        let Some(close_start) = art_content[template_start..]
            .find("</variant>")
            .map(|offset| template_start + offset)
        else {
            break;
        };
        collect_template_source_referenced_names(
            art_content[template_start..close_start].trim(),
            template_syntax,
            names,
        );
        cursor = close_start + "</variant>".len();
    }
}

fn is_variant_tag_boundary(next: Option<u8>) -> bool {
    matches!(next, Some(b' ' | b'\t' | b'\n' | b'\r' | b'>') | None)
}

fn collect_template_source_referenced_names(
    template: &str,
    template_syntax: TemplateSyntaxMode,
    names: &mut FxHashSet<CompactString>,
) {
    if template.is_empty() {
        return;
    }

    let allocator = Bump::new();
    let (root, _) = parse_with_options_and_template_syntax(
        &allocator,
        template,
        ParserOptions::default(),
        template_syntax,
    );
    names.extend(resolve_template_used_identifiers(&root).used_ids);
}
