use super::*;
use vize_l0::{Span, String as Text};
use vize_l4::module::ScopeId;

// Independently expand to a complete unscoped document, then traverse its
// generated lines once to add the validated scope and rebase each retained link.
fn separate(
    program: &ScssProgram<'_>,
    scope: ScopeId<'_>,
    trim: bool,
    mapped: bool,
) -> (Text, Vec<vize_l4::write::SpanLink>) {
    let mut doc = EmitDocument::new(mapped);
    scss::append(&mut doc, program);
    let (raw, mut links) = doc.into_parts();
    let mut out = Text::with_capacity(raw.len());
    let mut rest = links.iter_mut().peekable();
    let mut start = 0;
    for line in raw.split_inclusive('\n') {
        let end = start + line.len();
        let base = out.len();
        if let Some(selector) = line.strip_suffix(" {\n") {
            out.push_str(selector);
            out.push('[');
            out.push_str(scope.as_str());
            out.push_str("] {\n");
        } else if !(trim && line == "\n") {
            out.push_str(line);
        }
        while let Some(link) = rest.next_if(|l| (l.generated.start as usize) < end) {
            let first = base + (link.generated.start as usize - start);
            let last = base + (link.generated.end as usize - start);
            link.generated = Span::new(first as u32, last as u32);
        }
        start = end;
    }
    (out, links)
}

#[test]
#[ignore = "paired scoped SCSS lowering with exact complete CSS and links"]
fn fused_scoping_and_rule_output() {
    mark_installed();
    let scope = ScopeId::new("data-v-oracle").unwrap();
    for count in [1, 8, 48] {
        let mut text = "<template><p/></template><style lang=scss scoped>$v:4px;\n".to_owned();
        for i in 0..count {
            text.push_str(&format!(
                ".c{i}{{.child{{width:$v; padding:$v 0}} &.on{{margin:0}}}}\n"
            ));
        }
        text.push_str("</style>");
        let arena = Allocator::default();
        let session = NativeScssSession::new_scoped(&arena, &text, Default::default());
        let style = session
            .observation()
            .l1ir()
            .admitted()
            .unwrap()
            .styles()
            .next()
            .unwrap();
        let program = StyleLanguage::Scss
            .with_original_structure(style.block(), ScssProgram::lower)
            .unwrap();
        for (trim, mapped) in [(false, false), (false, true), (true, true)] {
            let render = |fused| {
                if fused {
                    let mut doc = EmitDocument::new(mapped);
                    scss::append_scoped(&mut doc, &program, scope, trim);
                    doc.into_parts()
                } else {
                    separate(&program, scope, trim, mapped)
                }
            };
            assert_eq!(render(false), render(true));
            let label = format!("scoped-rules rules={count} trim={trim} mapped={mapped}");
            let (old, new) = paired(&label, |fused| drop(black_box(render(fused))));
            assert!(new <= old, "{label} {old}/{new}");
        }
    }
}

#[test]
#[ignore = "paired complete scoped component, source maps and editor class queries"]
fn original_scoped_component_and_editor_share_the_same_styles() {
    mark_installed();
    for count in [1, 8, 48] {
        let mut source =
            "<script setup>let n=1</script><template>\r\n<p> a&#32; b {{n}}</p>\r\n</template>"
                .to_owned();
        for i in 0..count {
            source.push_str(&format!(
                "<style lang=scss scoped>$ink:red;.c{i}{{color:$ink;.child{{width:1px}}}}</style>"
            ));
        }
        super::super::prepared::measure_session("scoped-component", &source, count, |a, s, o| {
            NativeScssSession::new_scoped(a, s, o)
        });
    }
}
