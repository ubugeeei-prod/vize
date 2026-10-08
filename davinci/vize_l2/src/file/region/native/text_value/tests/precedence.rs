use super::*;
use vize_l1::{
    embed::{
        EmbedSource,
        syntax::{ProgramOptions, parse_program_once},
    },
    markup::{NativeTemplateGrammar, NativeTextValueError},
};

#[test]
fn original_ordinary_js_and_ts_programs_keep_script_refusal_and_whole_text_result() -> Test {
    for (source, typescript, grammar) in [
        (
            "<script>export default {};</script><template>&amp;lt;</template>",
            false,
            NativeTemplateGrammar::JavaScriptModule,
        ),
        (
            "<script lang='ts'>export default {};</script><template>&amp;lt;</template>",
            true,
            NativeTemplateGrammar::TypeScriptModule,
        ),
    ] {
        let arena = Allocator::default();
        let descriptor = Vue.observe_descriptor(
            &arena,
            source,
            DescriptorOptions {
                version: VueVersion::V3,
                dialect: VueDialect::Vue,
                template: SurfaceParseOptions::default(),
            },
        );
        let admitted = descriptor.admitted().map_err(|_| "ordinary Descriptor")?;
        let script = admitted.ordinary().ok_or("original ordinary block")?;
        let syntax = parse_program_once(
            &arena,
            EmbedSource::authored(source, script.block().span())
                .map_err(|_| "whole original script source")?,
            ProgramOptions::module(script.lang()),
        );
        let mut original = owner(&arena, source)?;
        original
            .ordinary_program(
                syntax
                    .admitted_program()
                    .ok_or("original ordinary Program")?,
            )
            .map_err(|_| "genuine ordinary attachment")?;
        let first;
        {
            let mut walk = original
                .begin()
                .map_err(|_| "ordinary empty-default begin")?;
            let selected = walk.selected();
            same(selected.grammar(), grammar)?;
            let child = selected
                .children()
                .next()
                .ok_or("original ordinary root Text")?;
            reset_observations();
            first = walk
                .root_text_value(child.reborrow())
                .err()
                .ok_or("ordinary prepared profile admitted")?;
            same(first.kind, Kind::TextValuePolicy(Policy::ScriptOrStyle))?;
            same(walk.root_text_value(child).err(), Some(first))?;
            same(observations(), 1)?;
            same(walk.complete().err(), Some(first))?;
        }
        let original = original.finish();
        same(original.view().err(), Some(first))?;
        let file = original.file().ok_or("whole refused ordinary File")?;
        check(!file.is_complete())?;
        same(file.units().len(), 1)?;
        same(file.units()[0].profile.typescript, typescript)?;
        same(file.exports().len(), 1)?;
        same(file.artifact().node_count(), 0)?;
        let [record] = file.native_text_values() else {
            return Err("one complete ordinary refusal row");
        };
        same(record.state(), State::Refused(first.kind))?;
        let value = record
            .observation()
            .ok_or("original prepared text retained")?;
        same(value.raw_text(), "&amp;lt;")?;
        same(value.source().text(), "&lt;")?;
        check(core::ptr::eq(value.source().authored_root(), source))?;
        same(
            value
                .source()
                .decode_map()
                .ok_or("whole original map")?
                .segments()
                .len(),
            2,
        )?;
    }
    Ok(())
}

#[test]
fn first_root_policy_refusal_survives_retry_completion_and_whole_owner_readback() -> Test {
    for (source, raw, decoded, policy) in [
        (
            "<template>a b</template>",
            "a b",
            "a b",
            Policy::RawWhitespace,
        ),
        (
            "<template>a&#32;b</template>",
            "a&#32;b",
            "a b",
            Policy::DecodedWhitespace,
        ),
        (
            "<template>&amp;lt;<!--tail--></template>",
            "&amp;lt;",
            "&lt;",
            Policy::RootExtent,
        ),
    ] {
        let arena = Allocator::default();
        let mut original = owner(&arena, source)?;
        let first;
        {
            let mut walk = original.begin().map_err(|_| "original begin")?;
            let selected = walk.selected();
            let child = selected.children().next().ok_or("original policy Text")?;
            reset_observations();
            first = walk
                .root_text_value(child.reborrow())
                .err()
                .ok_or("original policy admitted")?;
            same(first.kind, Kind::TextValuePolicy(policy))?;
            same(walk.root_text_value(child).err(), Some(first))?;
            same(observations(), 1)?;
            same(walk.root.facts.native_text_values.len(), 1)?;
            same(walk.root.facts.template_issues.len(), 1)?;
            same(walk.complete().err(), Some(first))?;
        }
        let original = original.finish();
        same(original.view().err(), Some(first))?;
        let file = original.file().ok_or("whole original refused File")?;
        check(!file.is_complete())?;
        same(file.artifact().node_count(), 0)?;
        let [record] = file.native_text_values() else {
            return Err("original refusal row replaced");
        };
        same(record.state(), State::Refused(first.kind))?;
        let value = record.observation().ok_or("whole policy result")?;
        same(value.raw_text(), raw)?;
        same(value.source().text(), decoded)?;
        check(core::ptr::eq(value.source().authored_root(), source))?;
    }
    Ok(())
}

#[test]
fn original_special_parent_failures_are_parked_before_root_extent_policy() -> Test {
    // These are the unchanged original L1 special-parent and inherited-v-pre controls.
    for (source, expected, span, inherited) in [
        (
            "<template><script>&amp;</script></template>",
            NativeTextValueError::RawTextParent,
            Span::new(18, 23),
            false,
        ),
        (
            "<template><stYle>&amp;</stYle></template>",
            NativeTextValueError::RawTextParent,
            Span::new(17, 22),
            false,
        ),
        (
            "<template><sTyLe>&amp;</sTyLe></template>",
            NativeTextValueError::RawTextParent,
            Span::new(17, 22),
            false,
        ),
        (
            "<template><StYlE>&amp;</StYlE></template>",
            NativeTextValueError::RawTextParent,
            Span::new(17, 22),
            false,
        ),
        (
            "<template><title>&amp;</title></template>",
            NativeTextValueError::RcDataParent,
            Span::new(17, 22),
            false,
        ),
        (
            "<template><teXtArEa>&amp;</teXtArEa></template>",
            NativeTextValueError::RcDataParent,
            Span::new(20, 25),
            false,
        ),
        (
            "<template><tExTaReA>&amp;</tExTaReA></template>",
            NativeTextValueError::RcDataParent,
            Span::new(20, 25),
            false,
        ),
        (
            "<template><TEXTAREA>&amp;</TEXTAREA></template>",
            NativeTextValueError::RcDataParent,
            Span::new(20, 25),
            false,
        ),
        (
            "<template><p v-pre>&amp;</p></template>",
            NativeTextValueError::Verbatim,
            Span::new(19, 24),
            false,
        ),
        (
            "<template><div v-pre><span>&amp;</span></div></template>",
            NativeTextValueError::Verbatim,
            Span::new(27, 32),
            true,
        ),
    ] {
        let arena = Allocator::default();
        let mut original = owner(&arena, source)?;
        let first;
        let parent_tag;
        {
            let mut walk = original
                .begin()
                .map_err(|_| "original special-parent begin")?;
            let selected = walk.selected();
            let root = selected
                .children()
                .next()
                .ok_or("original root Element")?
                .into_element()
                .ok_or("root Element")?;
            let parent = if inherited {
                root.children()
                    .next()
                    .ok_or("original v-pre descendant")?
                    .into_element()
                    .ok_or("descendant Element")?
            } else {
                root
            };
            parent_tag = parent.surface().tag();
            let child = parent
                .children()
                .next()
                .ok_or("original special-parent Text")?;
            reset_observations();
            first = walk
                .root_text_value(child.reborrow())
                .err()
                .ok_or("special-parent profile admitted")?;
            same(first.span, span)?;
            same(
                first.kind,
                Kind::TextValuePreparation {
                    span,
                    kind: expected,
                },
            )?;
            same(walk.root_text_value(child).err(), Some(first))?;
            same(observations(), 1)?;
            same(walk.complete().err(), Some(first))?;
        }
        let original = original.finish();
        same(original.view().err(), Some(first))?;
        let file = original.file().ok_or("whole special-parent refused File")?;
        check(!file.is_complete())?;
        same(file.artifact().node_count(), 0)?;
        let [record] = file.native_text_values() else {
            return Err("whole original L1 failure row");
        };
        same(record.state(), State::Refused(first.kind))?;
        check(record.observation().is_none())?;
        let failure = record
            .failure()
            .ok_or("actual parked preparation failure")?;
        same(failure.kind(), expected)?;
        same(failure.span(), Some(span))?;
        same(failure.ordinal(), 0)?;
        same(failure.parent_tag(), Some(parent_tag))?;
        same(
            failure.token().ok_or("complete original token")?.text,
            "&amp;",
        )?;
        check(core::ptr::eq(failure.block().root_source(), source))?;
        vize_l1::check_fidelity(&original.selected().component().carrier().tree)
            .map_err(|_| "full special-parent original source")?;
    }
    Ok(())
}
