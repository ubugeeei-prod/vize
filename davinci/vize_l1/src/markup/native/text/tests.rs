use super::{NativeRootTextError as Kind, NativeRootTextProfile};
use crate::{
    SurfaceChild, SurfaceParseOptions,
    container::{Vue, vue::DescriptorOptions},
    markup::NativeTemplateComponent,
};
use vize_l0::{
    Allocator,
    config::{VueDialect, VueVersion},
};

mod fixtures;
mod ownership;

fn selected<'a>(
    arena: &'a Allocator,
    source: &'a str,
) -> Result<NativeTemplateComponent<'a>, &'static str> {
    let descriptor = Vue.observe_descriptor(
        arena,
        source,
        DescriptorOptions {
            version: VueVersion::V3,
            dialect: VueDialect::Vue,
            template: SurfaceParseOptions::default(),
        },
    );
    NativeTemplateComponent::parse_in(
        arena,
        descriptor.admitted().map_err(|_| "original descriptor")?,
    )
    .map_err(|_| "original component")?
    .ok_or("selected original template")
}

#[test]
fn original_unicode_mixed_text_changes_only_derived_value_and_retains_full_span()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    let source = "<!--前雪🌸--><template>  雪🌸\t\r\n\x0c A  B </template>";
    let owner = selected(&arena, source)?;
    let child = owner.children().next().ok_or("original text")?;
    let SurfaceChild::Text(token) = child.surface() else {
        return Err("original text kind");
    };
    let raw = token.text;
    let pointer = child.surface();
    let before = arena.allocated_bytes();
    let receipt = owner
        .prepare_condensed_root_text(child)
        .map_err(|_| "original preparation")?;
    assert_eq!(receipt.profile(), NativeRootTextProfile::Vue3Condense);
    assert_eq!(receipt.content(), Some(" 雪🌸 A B "));
    assert_eq!(receipt.raw_text(), raw);
    assert!(core::ptr::eq(receipt.raw_text(), raw));
    assert_eq!(receipt.span().slice(source), raw);
    assert!(receipt.span().start > 20);
    assert!(arena.allocated_bytes() > before);
    let view = receipt
        .admitted_for_root_at(&owner, 0)
        .ok_or("same original root")?;
    assert!(core::ptr::eq(view.selected(), &owner));
    assert!(core::ptr::eq(view.receipt(), &receipt));
    assert!(core::ptr::eq(
        owner.children().next().ok_or("retained root")?.surface(),
        pointer
    ));
    crate::check_fidelity(&owner.component().carrier().tree).map_err(|_| "original fidelity")?;
    Ok(())
}

#[test]
fn unchanged_unicode_spaces_and_single_space_or_omission_allocate_no_arena_bytes()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    for (source, ordinal, expected, identity) in [
        (
            "<template>雪 \u{a0}\u{85}\u{1680}\u{2000}\u{2028}\u{2029}\u{3000}\u{feff} 花</template>",
            0,
            Some("雪 \u{a0}\u{85}\u{1680}\u{2000}\u{2028}\u{2029}\u{3000}\u{feff} 花"),
            true,
        ),
        ("<template><i/> <b/></template>", 1, Some(" "), true),
        ("<template><i/> \t\x0c<b/></template>", 1, Some(" "), false),
        ("<template> \t\r\n\x0c</template>", 0, None, false),
        ("<template><!--a--> \t<!--b--></template>", 1, None, false),
    ] {
        let owner = selected(&arena, source)?;
        let child = owner.children().nth(ordinal).ok_or("actual root slot")?;
        let before = arena.allocated_bytes();
        let receipt = owner
            .prepare_condensed_root_text(child)
            .map_err(|_| "original text receipt")?;
        assert_eq!(receipt.content(), expected, "{source}");
        assert_eq!(arena.allocated_bytes(), before, "{source}");
        if identity {
            assert!(core::ptr::eq(
                receipt.content().ok_or("identity value")?,
                receipt.raw_text()
            ));
        }
        assert_eq!(receipt.span().slice(source), receipt.raw_text());
        crate::check_fidelity(&owner.component().carrier().tree).map_err(|_| "fidelity")?;
    }
    Ok(())
}

#[test]
fn only_original_ascii_runs_condense_and_unicode_whitespace_is_never_trimmed()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    for (source, expected) in [
        ("<template> \u{a0}  </template>", " \u{a0} "),
        ("<template> \u{85}\t </template>", " \u{85} "),
        (
            "<template>\u{3000}\t\t雪 \u{feff}</template>",
            "\u{3000} 雪 \u{feff}",
        ),
        ("<template> \r\n雪\x0c 花\t </template>", " 雪 花 "),
    ] {
        let owner = selected(&arena, source)?;
        let receipt = owner
            .prepare_condensed_root_text(owner.children().next().ok_or("root text")?)
            .map_err(|_| "condensed original text")?;
        assert_eq!(receipt.content(), Some(expected), "{source}");
        assert_eq!(receipt.span().slice(source), receipt.raw_text());
        assert_eq!(owner.children().len(), 1);
    }
    Ok(())
}
