use vize_croquis::Croquis;
use vize_l0::{Allocator, String};

use crate::l2::L2Refusal;
use crate::{JsxLang, lower_source};

use super::super::{VdomCompatOptions, VdomCompileOptions, compile_root_to_vdom};

#[test]
fn component_option_event_modifiers_emit_from_l2() {
    let cases = [
        "const A = () => <B onClickCapture={h} />;",
        "const A = () => <B onClickOnce={h} />;",
        "const A = () => <B onInputPassiveCapture={h} />;",
    ];

    for source in cases {
        let l2 = compile_case(source, EmitRoute::ForceL2);
        let relief = compile_case(source, EmitRoute::ForceRelief);

        assert_eq!(l2.preamble, relief.preamble, "{source}");
        assert_eq!(l2.code, relief.code, "{source}");
    }
}

#[derive(Clone, Copy)]
enum EmitRoute {
    ForceL2,
    ForceRelief,
}

struct Output {
    preamble: String,
    code: String,
}

fn compile_case(source: &str, route: EmitRoute) -> Output {
    let allocator = Allocator::new();
    let mut lowered = lower_source(&allocator, allocator.as_oxc(), source, JsxLang::Jsx);
    assert!(!lowered.has_errors(), "{:?}", lowered.diagnostics);

    let analysis: &Croquis = allocator.alloc_owned(lowered.analysis);
    let mut root = lowered.roots.pop().expect("one JSX root");
    assert!(lowered.roots.is_empty(), "expected exactly one JSX root");

    match route {
        EmitRoute::ForceL2 => {
            let l2 = root
                .l2
                .as_ref()
                .expect("component event option modifiers project to L2");
            assert!(super::root_is_supported(l2));
            root.root.children.clear();
        }
        EmitRoute::ForceRelief => root.l2 = Err(L2Refusal::UnsupportedChild),
    }

    let mut diagnostics = Vec::new();
    let component = compile_root_to_vdom(
        &allocator,
        root,
        analysis,
        false,
        &VdomCompileOptions::default(),
        VdomCompatOptions::default(),
        &mut diagnostics,
        source,
    );
    assert!(diagnostics.is_empty(), "{diagnostics:?}");

    Output {
        preamble: component.preamble,
        code: component.code,
    }
}
