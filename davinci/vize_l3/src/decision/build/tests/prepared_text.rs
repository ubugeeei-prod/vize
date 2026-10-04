//! New source custody cannot bypass any existing original native entry.
use crate::decision::{DecisionBuildError as Error, native, policy::TargetPolicy, ssr, vapor};
use vize_l0::{
    Allocator,
    config::{VueDialect, VueVersion},
};
use vize_l1::{
    SurfaceParseOptions,
    container::{Vue, vue::DescriptorOptions},
    markup::NativeTemplateComponent,
};
use vize_l2::lang::js::{NativeTemplateFile, NativeTemplateOwner};
type Test = Result<(), &'static str>;
fn check(value: bool) -> Test {
    if value {
        Ok(())
    } else {
        Err("explicit original prepared-text profile")
    }
}
fn completed<'a>(
    arena: &'a Allocator,
    source: &'a str,
    prepared: bool,
) -> Result<NativeTemplateFile<'a>, &'static str> {
    let descriptor = Vue.observe_descriptor(
        arena,
        source,
        DescriptorOptions {
            version: VueVersion::V3,
            dialect: VueDialect::Vue,
            template: SurfaceParseOptions::default(),
        },
    );
    let selected =
        NativeTemplateComponent::parse_in(arena, descriptor.admitted().map_err(|_| "Descriptor")?)
            .map_err(|_| "original parse")?
            .ok_or("template")?;
    let mut original = NativeTemplateOwner::new(selected).map_err(|_| "owner")?;
    {
        let mut walk = original.begin().map_err(|_| "begin")?;
        for child in walk.selected().children() {
            if prepared {
                walk.root_text_value(child).map_err(|_| "prepared event")?;
            } else {
                let receipt = walk
                    .selected()
                    .prepare_condensed_root_text(child)
                    .map_err(|_| "original condense")?;
                walk.root_text(&receipt).map_err(|_| "unchanged old root")?;
            }
        }
        walk.complete().map_err(|_| "complete")?;
    }
    Ok(original.finish())
}

#[test]
fn complete_original_prepared_text_view_is_refused_by_each_existing_native_target_entry() -> Test {
    let arena = Allocator::default();
    let original = completed(&arena, "<template>&amp;lt;</template>", true)?;
    let file = original.file().ok_or("complete File")?;
    check(file.is_complete() && file.native_text_values().len() == 1)?;
    check(
        native::build_native_dom_file_decisions(original.view().map_err(|_| "view")?).err()
            == Some(Error::PreparedTextProfile),
    )?;
    check(
        ssr::build_native_ssr_file_decisions(original.view().map_err(|_| "view")?).err()
            == Some(ssr::NativeSsrBuildError::Decision(
                Error::PreparedTextProfile,
            )),
    )?;
    check(
        vapor::build_native_vapor_file_decisions(original.view().map_err(|_| "view")?).err()
            == Some(Error::PreparedTextProfile),
    )?;
    // All genuine context constructors converge on this same original-file
    // guard, including optional SSR/setup and Vapor/setup contexts. The new
    // L2 method refuses those actual script selections before attachment.
    for policy in [TargetPolicy::Dom, TargetPolicy::Ssr, TargetPolicy::Vapor] {
        check(
            super::super::build_with_original(
                file,
                policy,
                &super::super::LiteralExpressions,
                &super::super::NoReads,
            )
            .err()
                == Some(Error::PreparedTextProfile),
        )?;
    }
    Ok(())
}

#[test]
fn existing_condensed_owner_and_all_generic_none_routes_keep_their_exact_authority() -> Test {
    let arena = Allocator::default();
    let old = completed(&arena, "<template>plain</template>", false)?;
    check(
        old.file()
            .ok_or("old File")?
            .native_text_values()
            .is_empty(),
    )?;
    check(native::build_native_dom_file_decisions(old.view().map_err(|_| "old view")?).is_ok())?;
    check(ssr::build_native_ssr_file_decisions(old.view().map_err(|_| "old view")?).is_ok())?;
    check(vapor::build_native_vapor_file_decisions(old.view().map_err(|_| "old view")?).is_ok())?;
    let new = completed(&arena, "<template>&amp;lt;</template>", true)?;
    let file = new.file().ok_or("new File")?;
    // Generic diagnostic APIs intentionally retain their neutral None policy;
    // none can manufacture a genuine original prepared-template capability.
    check(crate::decision::build_dom_file_decisions(file).is_ok())?;
    check(ssr::build_ssr_file_decisions(file).is_ok())?;
    check(vapor::build_vapor_file_decisions(file).is_ok())?;
    for policy in [TargetPolicy::Dom, TargetPolicy::Ssr, TargetPolicy::Vapor] {
        check(crate::decision::build_decisions(file.artifact(), policy).is_ok())?;
    }
    Ok(())
}
