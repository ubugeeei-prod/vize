//! The MoonBit dialect without a toolchain: projection, span links,
//! capability answers, and the diagnostic mapping replayed from the
//! committed `moonc` answers. `moonc_e2e` proves those answers are what
//! the pinned toolchain says today.

#![expect(clippy::string_slice, reason = "tests assert by panicking")]

mod support;

use std::fmt::Write as _;

use vize_dialect_moonbit::diagnostic::Origin;
use vize_dialect_moonbit::dialect::MoonBitDialect;
use vize_dialect_moonbit::host::Replay;
use vize_dialect_moonbit::projection::{Role, project};
use vize_dialect_moonbit::render::render;
use vize_dialect_moonbit::sfc::{SfcError, split};
use vize_s0::{Allocator, Span};
use vize_s2::expr::ExprRef;
use vize_s2::expr::capability::ExprDialect;

use support::{FIXTURES, golden, pinned_toolchain, read};

#[test]
fn projection_is_pinned_and_every_link_is_a_verbatim_copy() {
    for name in FIXTURES {
        let source = read(name, ".vue");
        let allocator = Allocator::new();
        let sfc = split(&source).expect("a MoonBit SFC");
        let projection = project(&allocator, &sfc, &[name, ".vue"].concat());
        golden(name, ".vue.mbt", &projection.text);
        assert!(
            projection.unsupported.is_empty(),
            "{name}: {:?}",
            projection.unsupported
        );
        assert_eq!(projection.links[0].role, Role::Script);
        assert_eq!(projection.links[0].source, sfc.script.span());
        for link in &projection.links {
            assert_eq!(
                slice(&projection.text, link.generated),
                slice(&source, link.source),
                "{name}: {link:?}"
            );
        }
        for (index, position) in projection.positions.iter().enumerate() {
            let link = projection.links[index + 1];
            assert_eq!(link.role, Role::Expression(index));
            assert_eq!(link.source, position.expr.span);
            assert!(position.statement.start <= link.generated.start);
            assert!(link.generated.end <= position.statement.end);
        }
    }
}

#[test]
fn capability_answers_are_pinned_per_position() {
    for name in FIXTURES {
        let source = read(name, ".vue");
        let allocator = Allocator::new();
        let projection = project(&allocator, &split(&source).unwrap(), "App.vue");
        let mut facts = vize_s0::String::default();
        for position in &projection.positions {
            let expr = ExprRef::Foreign(position.expr);
            let mut names = Vec::new();
            MoonBitDialect
                .enumerate_bindings(expr, &mut |name| names.push(vize_s0::String::from(name)));
            let span = expr.span();
            let _ = writeln!(
                facts,
                "{:?} {}..{} `{}` bindings={names:?} exact={} constant={} inner0..1={:?}",
                position.kind,
                span.start,
                span.end,
                expr.source(),
                MoonBitDialect.bindings_are_exact(expr),
                MoonBitDialect.is_constant(expr),
                MoonBitDialect.map_span(expr, Span::new(0, 1)),
            );
        }
        golden(name, ".facts", &facts);
    }
}

#[test]
fn replayed_answers_map_into_the_template_exactly() {
    for name in FIXTURES {
        let source = read(name, ".vue");
        let allocator = Allocator::new();
        let file_name = [name, ".vue"].concat();
        let mut host = Replay::new(&pinned_toolchain(), &read(name, ".moonc.jsonl"));
        let checked = vize_dialect_moonbit::check(&allocator, &source, &file_name, &mut host)
            .expect("the replay answers");
        assert!(
            checked
                .diagnostics
                .iter()
                .all(|mapped| mapped.origin != Origin::Projection),
            "{name}: a diagnostic points into the generated scaffolding"
        );
        let rendered = render(
            &source,
            &file_name,
            &checked.projection,
            &checked.diagnostics,
        );
        golden(name, ".diagnostics", &rendered);
    }
}

#[test]
fn only_moonbit_script_blocks_select_the_dialect() {
    let ts = "<script setup lang=\"ts\">const a = 1</script><template>{{ a }}</template>";
    assert_eq!(
        split(ts).unwrap_err(),
        SfcError::NotMoonBit(Some("ts".into()))
    );
    let bare = "<script setup>const a = 1</script><template>{{ a }}</template>";
    assert_eq!(split(bare).unwrap_err(), SfcError::NotMoonBit(None));
    let pug = "<script setup lang=\"mbt\">let a = 1</script><template lang=\"pug\">p</template>";
    assert_eq!(split(pug).unwrap_err(), SfcError::TemplateNotInline);
    let none = "<script setup lang=\"moonbit\">let a = 1</script>";
    assert_eq!(split(none).unwrap_err(), SfcError::NoTemplate);
    let mbt = "<script setup lang=\"mbt\">let a = 1</script><template>{{ a }}</template>";
    assert_eq!(split(mbt).unwrap().template.source(), "{{ a }}");
}

#[test]
fn positions_outside_the_subset_are_reported_not_dropped() {
    let source = "<script setup lang=\"moonbit\">let a : Ref[Int] = { val: 1 }</script>\n\
                  <template><input v-model=\"a.val\"><p v-for=\"(x, i, n) in xs\">{{ x }}</p></template>";
    let allocator = Allocator::new();
    let projection = project(&allocator, &split(source).unwrap(), "App.vue");
    let what: Vec<_> = projection.unsupported.iter().map(|u| u.what).collect();
    assert_eq!(what, ["ui.model", "ui.for (third alias)"]);
    for unsupported in &projection.unsupported {
        assert!(!slice(source, unsupported.span).is_empty());
    }
}

fn slice(text: &str, span: Span) -> &str {
    &text[span.start as usize..span.end as usize]
}
