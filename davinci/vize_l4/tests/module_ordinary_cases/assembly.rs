//! Assembly provider laws; prepared render fragments grant no target/product admission.

use super::{Observed, check, equal};
use vize_l0::{Allocator, Span};
use vize_l4::module::{
    ModuleParts, RenderPlacement, RenderProperty, ScriptPart, assemble,
    ordinary::emit_ordinary_empty, setup::COMPONENT_BINDING,
};
use vize_l4::runtime::Vocabulary;
use vize_l4::write::{NoLinks, Recorded, Writer};

const NO_HELPERS: Vocabulary = Vocabulary { modules: &[] };

#[test]
fn genuine_script_only_module_copies_leading_object_and_tail_once_without_setup()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    let source = "<script>/*lead雪*/\r\nexport default { /*inside*/ }; //tail\n</script>";
    let observed = Observed::new(&arena, source)?;
    let file = observed.file(&arena)?;
    let view = observed.checked(&file)?;
    let emitted = assemble(ModuleParts {
        vocabulary: &NO_HELPERS,
        prelude: None,
        script: Some(ScriptPart::Body(
            emit_ordinary_empty::<Recorded>(&view).map_err(|_| "script")?,
        )),
        render: None,
        placement: RenderPlacement::None,
        component: COMPONENT_BINDING,
    })
    .map_err(|_| "recorded assembly")?;
    let plain = assemble(ModuleParts {
        vocabulary: &NO_HELPERS,
        prelude: None,
        script: Some(ScriptPart::Body(
            emit_ordinary_empty::<NoLinks>(&view).map_err(|_| "script")?,
        )),
        render: None,
        placement: RenderPlacement::None,
        component: COMPONENT_BINDING,
    })
    .map_err(|_| "unrecorded assembly")?;
    equal(
        emitted.text.as_str(),
        "/*lead雪*/\r\nconst _sfc_main = { /*inside*/ }; //tail\nexport default _sfc_main\n",
    )?;
    equal(emitted.text.as_str(), plain.text.as_str())?;
    check(emitted.helpers.is_empty() && plain.helpers.is_empty())?;
    let leading = Span::new(view.source().start(), view.statement_span().start);
    let tail = Span::new(view.object_span().start, view.source().end());
    let links = emitted.links.links();
    equal(links.len(), 2)?;
    equal(links[0].authored, leading)?;
    equal(links[1].authored, tail)?;
    for link in links {
        equal(
            source.get(link.authored.start as usize..link.authored.end as usize),
            emitted
                .text
                .get(link.generated.start as usize..link.generated.end as usize),
        )?;
        check(link.name.is_none() && link.segment)?;
    }
    // Every copied original byte has one link; the certified export prefix has none.
    for byte in view.source().start()..view.source().end() {
        let copies = links
            .iter()
            .filter(|link| link.authored.start <= byte && byte < link.authored.end)
            .count();
        equal(
            copies,
            usize::from(byte < view.statement_span().start || byte >= view.object_span().start),
        )?;
    }
    let declaration_start = leading.end - leading.start;
    equal(links[0].generated, Span::new(0, declaration_start))?;
    equal(
        links[1].generated.start as usize,
        declaration_start as usize + "const _sfc_main = ".len(),
    )?;
    equal(
        emitted.text.get(links[1].generated.end as usize..),
        Some("export default _sfc_main\n"),
    )?;
    check(core::ptr::eq(view.file(), &file))?;
    check(core::ptr::eq(view.source().root_source(), source))?;
    check(!plain.into_document().is_recording())?;
    Ok(())
}

#[test]
fn genuine_ts_script_attaches_a_prepared_render_without_linking_synthesized_module_bytes()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    let source = "<script lang='ts'>'use strict';\r\n;export /*omit*/ default /*omit*/ { /*kept*/ };\r\n//tail</script>";
    let observed = Observed::new(&arena, source)?;
    let file = observed.file(&arena)?;
    let view = observed.checked(&file)?;
    check(file.units()[0].profile.typescript)?;
    // This is a deliberately prepared assembler fragment, not native target output.
    let mut render: Writer<Recorded> = Writer::default();
    render.push("function _sfc_render() { return 'prepared' }");
    let emitted = assemble(ModuleParts {
        vocabulary: &NO_HELPERS,
        prelude: None,
        script: Some(ScriptPart::Body(
            emit_ordinary_empty::<Recorded>(&view).map_err(|_| "script")?,
        )),
        render: Some(render),
        placement: RenderPlacement::Function {
            binding: "_sfc_render",
            property: RenderProperty::Render,
        },
        component: COMPONENT_BINDING,
    })
    .map_err(|_| "recorded assembly")?;
    let mut render: Writer<NoLinks> = Writer::default();
    render.push("function _sfc_render() { return 'prepared' }");
    let plain = assemble(ModuleParts {
        vocabulary: &NO_HELPERS,
        prelude: None,
        script: Some(ScriptPart::Body(
            emit_ordinary_empty::<NoLinks>(&view).map_err(|_| "script")?,
        )),
        render: Some(render),
        placement: RenderPlacement::Function {
            binding: "_sfc_render",
            property: RenderProperty::Render,
        },
        component: COMPONENT_BINDING,
    })
    .map_err(|_| "unrecorded assembly")?;
    equal(
        emitted.text.as_str(),
        concat!(
            "'use strict';\r\n;const _sfc_main = { /*kept*/ };\r\n//tail\n;\n",
            "function _sfc_render() { return 'prepared' }\n",
            "_sfc_main.render = _sfc_render\nexport default _sfc_main\n",
        ),
    )?;
    equal(emitted.text.as_str(), plain.text.as_str())?;
    check(emitted.helpers.is_empty() && plain.helpers.is_empty())?;
    equal(observed.syntax.comments().count(), 4)?;
    let links = emitted.links.links();
    equal(links.len(), 2)?;
    equal(
        links[0].authored,
        Span::new(view.source().start(), view.statement_span().start),
    )?;
    equal(
        links[1].authored,
        Span::new(view.object_span().start, view.source().end()),
    )?;
    for link in links {
        equal(
            source.get(link.authored.start as usize..link.authored.end as usize),
            emitted
                .text
                .get(link.generated.start as usize..link.generated.end as usize),
        )?;
        check(link.name.is_none() && link.segment)?;
    }
    // Generated delimiter, prepared render, attachment and final default stay unlinked.
    equal(
        emitted.text.get(links[1].generated.end as usize..),
        Some(concat!(
            "\n;\nfunction _sfc_render() { return 'prepared' }\n",
            "_sfc_main.render = _sfc_render\nexport default _sfc_main\n",
        )),
    )?;
    check(!plain.into_document().is_recording())?;
    Ok(())
}

#[test]
fn absent_leading_gap_mints_no_empty_link_and_leaves_generated_binding_and_default_unlinked()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    for (terminator, prepared_render) in [(";", false), ("", false), ("", true)] {
        let source = format!("<script>export default {{}}{terminator}</script>");
        let observed = Observed::new(&arena, &source)?;
        let file = observed.file(&arena)?;
        let view = observed.checked(&file)?;
        // The optional function is a prepared assembler input, not native target output.
        let render = prepared_render.then(|| {
            let mut render: Writer<Recorded> = Writer::default();
            render.push("function _sfc_render() { return 'prepared' }");
            render
        });
        let placement = if prepared_render {
            RenderPlacement::Function {
                binding: "_sfc_render",
                property: RenderProperty::Render,
            }
        } else {
            RenderPlacement::None
        };
        let emitted = assemble(ModuleParts {
            vocabulary: &NO_HELPERS,
            prelude: None,
            script: Some(ScriptPart::Body(
                emit_ordinary_empty::<Recorded>(&view).map_err(|_| "script")?,
            )),
            render,
            placement,
            component: COMPONENT_BINDING,
        })
        .map_err(|_| "recorded assembly")?;
        let render = prepared_render.then(|| {
            let mut render: Writer<NoLinks> = Writer::default();
            render.push("function _sfc_render() { return 'prepared' }");
            render
        });
        let plain = assemble(ModuleParts {
            vocabulary: &NO_HELPERS,
            prelude: None,
            script: Some(ScriptPart::Body(
                emit_ordinary_empty::<NoLinks>(&view).map_err(|_| "script")?,
            )),
            render,
            placement,
            component: COMPONENT_BINDING,
        })
        .map_err(|_| "unrecorded assembly")?;
        let suffix = if prepared_render {
            concat!(
                "\n;\nfunction _sfc_render() { return 'prepared' }\n",
                "_sfc_main.render = _sfc_render\nexport default _sfc_main\n",
            )
        } else {
            "\nexport default _sfc_main\n"
        };
        let object_tail = format!("{{}}{terminator}");
        let expected = format!("const _sfc_main = {object_tail}{suffix}");
        equal(emitted.text.as_str(), expected.as_str())?;
        equal(emitted.text.as_str(), plain.text.as_str())?;
        let [link] = emitted.links.links() else {
            return Err("single retained object/tail");
        };
        equal(
            link.authored,
            Span::new(view.object_span().start, view.source().end()),
        )?;
        equal(
            source.get(link.authored.start as usize..link.authored.end as usize),
            Some(object_tail.as_str()),
        )?;
        equal(
            emitted.text.get(..link.generated.start as usize),
            Some("const _sfc_main = "),
        )?;
        equal(
            emitted
                .text
                .get(link.generated.start as usize..link.generated.end as usize),
            Some(object_tail.as_str()),
        )?;
        // Declaration, ASI delimiter, prepared render, attachment and export are unlinked.
        equal(
            emitted.text.get(link.generated.end as usize..),
            Some(suffix),
        )?;
        check(link.generated.start < link.generated.end && link.name.is_none() && link.segment)?;
        check(emitted.helpers.is_empty() && plain.helpers.is_empty())?;
        check(!plain.into_document().is_recording())?;
    }
    Ok(())
}
