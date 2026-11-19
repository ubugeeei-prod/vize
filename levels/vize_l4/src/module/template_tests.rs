//! Template assembly uses the same checked preamble and linked append path.

use vize_l0::Span;

use super::{AssemblyError, assemble_template};
use crate::runtime::{Helper, Runtime, vocabulary};
use crate::write::{LinkSink, NoLinks, Recorded, Writer};

fn fragment<L: LinkSink>() -> Writer<L> {
    let mut writer = Writer::default();
    writer.push("function render() { return _toDisplayString(");
    writer.push_named("msg", Span::new(0, 3), "msg");
    writer.push(") + ");
    writer.push_linked("'雪'", Span::new(4, 9));
    writer.push(" }");
    writer.use_helper(
        vocabulary(Runtime::VueDom)
            .helper("toDisplayString")
            .unwrap(),
    );
    writer
}

#[test]
fn template_assembly_retains_named_and_anonymous_links_after_late_imports() {
    let runtime = vocabulary(Runtime::VueDom);
    let recorded = assemble_template(fragment::<Recorded>(), runtime).unwrap();
    let plain = assemble_template(fragment::<NoLinks>(), runtime).unwrap();
    assert_eq!(recorded.text, plain.text);
    assert_eq!(recorded.helpers, plain.helpers);
    assert_eq!(
        recorded.text.as_str(),
        concat!(
            "import { toDisplayString as _toDisplayString } from \"vue\"\n\n",
            "export function render() { return _toDisplayString(msg) + '雪' }"
        )
    );
    let document = recorded.into_document();
    assert_eq!(document.links().len(), 2);
    let named = document.links().first().unwrap();
    assert_eq!(named.authored, Span::new(0, 3));
    assert_eq!(named.name.as_deref(), Some("msg"));
    assert_eq!(
        document
            .as_str()
            .get(named.generated.start as usize..named.generated.end as usize),
        Some("msg")
    );
    let literal = document.links().get(1).unwrap();
    assert_eq!(literal.authored, Span::new(4, 9));
    assert!(literal.name.is_none());
    assert_eq!(
        document
            .as_str()
            .get(literal.generated.start as usize..literal.generated.end as usize),
        Some("'雪'")
    );
    assert!(plain.into_document().links().is_empty());
}

#[test]
fn template_assembly_refuses_unknown_helpers_for_both_link_sinks() {
    let helper = Helper::from_index(127).unwrap();
    let mut recorded = Writer::<Recorded>::default();
    recorded.use_helper(helper);
    let mut plain = Writer::<NoLinks>::default();
    plain.use_helper(helper);
    let runtime = vocabulary(Runtime::VueDom);
    assert_eq!(
        assemble_template(recorded, runtime).unwrap_err(),
        AssemblyError::UnknownHelper(helper)
    );
    assert_eq!(
        assemble_template(plain, runtime).unwrap_err(),
        AssemblyError::UnknownHelper(helper)
    );
}

#[test]
fn template_assembly_refuses_an_ssr_local_index_absent_from_the_dom_vocabulary() {
    let helper = vocabulary(Runtime::VueServerRenderer)
        .helper("useCssVars")
        .unwrap();
    assert!(vocabulary(Runtime::VueDom).name(helper).is_none());
    let mut render = Writer::<NoLinks>::default();
    render.use_helper(helper);
    assert_eq!(
        assemble_template(render, vocabulary(Runtime::VueDom)).unwrap_err(),
        AssemblyError::UnknownHelper(helper)
    );
}
