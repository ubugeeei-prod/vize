use crate::{check, equal};
use vize_l0::{Span, id::NodeId};
use vize_l1::{
    ElementClose, SurfaceChild,
    markup::{NativeChild, NativeChildren},
};
use vize_l2::{
    lang::js::NativeTemplateFile,
    op::{ElementOp, Namespace, Op},
};
use vize_l3::decision::{StaticLevel, dom::DomChanges, native::NativeTemplateDomAnalysis};
use vize_l4::write::EmitDocument;

pub(crate) fn inspect<'owner, 'a>(
    id: &str,
    original: &'owner NativeTemplateFile<'a>,
    analysis: &NativeTemplateDomAnalysis<'owner, 'a>,
    document: &EmitDocument,
) -> Result<(u32, u32), &'static str> {
    let file = original.file().ok_or("original File")?;
    check(file.is_complete())?;
    check(core::ptr::eq(analysis.owner(), original))?;
    check(core::ptr::eq(analysis.file(), file))?;
    check(core::ptr::eq(analysis.artifact(), file.artifact()))?;
    check(core::ptr::eq(
        file.artifact().source(),
        original.selected().component().block().root_source(),
    ))?;
    check(analysis.tables().controls.is_empty())?;
    check(analysis.dom().ok_or("DOM facts")?.unsupported().is_empty())?;
    let expected = header_spans(id)?;
    let mut counts = (0, 0);
    children(
        original.selected().children(),
        &file.artifact().root().ops,
        analysis,
        document,
        expected,
        &mut counts,
    )?;
    equal(counts.1 as usize, expected.len())?;
    equal(analysis.tables().nodes.len(), counts.0 as usize)?;
    let next = NodeId::from_index(counts.0).ok_or("next node")?;
    check(analysis.tables().nodes.get(next).is_none())?;
    check(analysis.dom().ok_or("DOM facts")?.node(next).is_none())?;
    for link in document.links() {
        check(
            file.artifact()
                .source()
                .get(link.authored.start as usize..link.authored.end as usize)
                .is_some(),
        )?;
        check(
            document
                .as_str()
                .get(link.generated.start as usize..link.generated.end as usize)
                .is_some(),
        )?;
        check(link.name.is_none())?;
    }
    Ok(counts)
}

fn children<'owner, 'a>(
    original: NativeChildren<'owner, 'a>,
    canonical: &'owner [Op<'a>],
    analysis: &NativeTemplateDomAnalysis<'owner, 'a>,
    document: &EmitDocument,
    expected: &[Span],
    counts: &mut (u32, u32),
) -> Result<(), &'static str> {
    equal(original.len(), canonical.len())?;
    for (ordinal, (original, canonical)) in original.zip(canonical).enumerate() {
        equal(original.ordinal(), ordinal)?;
        check(core::ptr::eq(
            original.component(),
            analysis.owner().selected().component(),
        ))?;
        let node = NodeId::from_index(counts.0).ok_or("original node")?;
        counts.0 += 1;
        let dom = analysis.dom().ok_or("DOM facts")?;
        let fact = dom.node(node).ok_or("original DOM node")?;
        check(core::ptr::eq(fact.op(), canonical))?;
        equal(fact.changes, DomChanges::default())?;
        check(fact.dynamic_property_bindings.is_empty())?;
        check(dom.binding(node).is_none() && dom.conditional(node).is_none())?;
        check(analysis.expression(node).is_none())?;
        let row = analysis
            .tables()
            .nodes
            .get(node)
            .ok_or("original node row")?;
        check(row.dynamic_bindings.is_empty() && row.control.is_none())?;
        match (original.surface(), canonical) {
            (SurfaceChild::Element(_), Op::Element(element)) => {
                equal(row.static_level, StaticLevel::Static)?;
                equal(row.output_level, StaticLevel::Static)?;
                element_projection(original, element, analysis, document, expected, counts)?;
            }
            (SurfaceChild::Text(token), Op::Text(text)) => {
                check(core::ptr::eq(text.content, token.text))?;
                equal(
                    text.span,
                    original
                        .component()
                        .block()
                        .span_of(token.text)
                        .ok_or("original text span")?,
                )?;
                equal(row.output_level, StaticLevel::Static)?;
            }
            (SurfaceChild::Comment(token), Op::Comment(comment)) => {
                let body = token
                    .text
                    .strip_prefix("<!--")
                    .and_then(|text| text.strip_suffix("-->"))
                    .ok_or("original comment framing")?;
                check(core::ptr::eq(comment.content, body))?;
                equal(
                    comment.span,
                    original
                        .component()
                        .block()
                        .span_of(token.text)
                        .ok_or("original comment span")?,
                )?;
                equal(row.output_level, StaticLevel::Dynamic)?;
            }
            _ => return Err("same original ordered operation"),
        }
    }
    Ok(())
}

fn element_projection<'owner, 'a>(
    original: NativeChild<'owner, 'a>,
    element: &'owner ElementOp<'a>,
    analysis: &NativeTemplateDomAnalysis<'owner, 'a>,
    document: &EmitDocument,
    expected: &[Span],
    counts: &mut (u32, u32),
) -> Result<(), &'static str> {
    let original = original.into_element().ok_or("actual original Element")?;
    check(core::ptr::eq(element.tag, original.surface().tag()))?;
    equal(element.namespace, Namespace::Html)?;
    check(element.bindings.is_empty())?;
    let block = original.component().block();
    let start = block
        .span_of(original.surface().open.lt_name.text)
        .ok_or("opening extent")?
        .start;
    let end = match &original.surface().close {
        ElementClose::Present(close) => block.span_of(close.gt.text),
        ElementClose::NotExpected => block.span_of(original.surface().open.gt.text),
        _ => return Err("genuine closing extent"),
    }
    .ok_or("closing span")?
    .end;
    equal(element.span, Span::new(start, end))?;
    equal(original.attributes().len(), element.attributes.len())?;
    for (ordinal, (attribute, canonical)) in
        original.attributes().zip(&element.attributes).enumerate()
    {
        equal(attribute.ordinal(), ordinal)?;
        check(core::ptr::eq(attribute.component(), original.component()))?;
        check(core::ptr::eq(attribute.element(), original.surface()))?;
        check(core::ptr::eq(
            attribute.surface(),
            original
                .surface()
                .open
                .attrs
                .get(ordinal)
                .ok_or("full original Attribute")?,
        ))?;
        check(core::ptr::eq(canonical.name, attribute.surface().name.text))?;
        let name = block
            .span_of(attribute.surface().name.text)
            .ok_or("original name span")?;
        let end = if let Some(value) = &attribute.surface().value {
            let token = value.close_quote.as_ref().unwrap_or(&value.content);
            block.span_of(token.text).ok_or("original value end")?.end
        } else {
            name.end
        };
        equal(canonical.span, Span::new(name.start, end))?;
        equal(
            canonical.span,
            *expected
                .get(counts.1 as usize)
                .ok_or("pinned full attribute extent")?,
        )?;
        match (canonical.value, &attribute.surface().value) {
            (None, None) => {}
            (Some(value), Some(original)) => check(core::ptr::eq(value, original.content.text))?,
            _ => return Err("bare or present original value"),
        }
        attribute_links(document, canonical.name, canonical.value, canonical.span)?;
        counts.1 += 1;
    }
    children(
        original.children(),
        &element.children.ops,
        analysis,
        document,
        expected,
        counts,
    )
}

// Independent byte extents of the six pinned authored sources, including UTF-8.
fn header_spans(id: &str) -> Result<&'static [Span], &'static str> {
    const ORDERED: &[Span] = &[
        Span::new(19, 35),
        Span::new(36, 42),
        Span::new(43, 56),
        Span::new(57, 71),
        Span::new(72, 93),
        Span::new(101, 109),
        Span::new(117, 129),
    ];
    const UNICODE: &[Span] = &[Span::new(13, 38), Span::new(39, 55), Span::new(56, 63)];
    const BARE: &[Span] = &[Span::new(15, 21), Span::new(22, 35), Span::new(36, 44)];
    const NESTED: &[Span] = &[Span::new(15, 26), Span::new(33, 44), Span::new(59, 70)];
    match id {
        "ordered-headers" => Ok(ORDERED),
        "unicode-escape" => Ok(UNICODE),
        "bare-empty" => Ok(BARE),
        "nested" => Ok(NESTED),
        "text-comment" | "empty" => Ok(&[]),
        _ => Err("reviewed original source extents"),
    }
}

fn attribute_links(
    document: &EmitDocument,
    name: &str,
    value: Option<&str>,
    span: Span,
) -> Result<(), &'static str> {
    let key = match name {
        "id" | "hidden" | "disabled" | "title" => name.to_owned(),
        "data-empty" | "data-count" | "aria-label" | "data-note" | "data-id" | "名" => {
            serde_json::to_string(name).map_err(|_| "quoted property key")?
        }
        _ => return Err("reviewed fixture property"),
    };
    let value = serde_json::to_string(value.unwrap_or("")).map_err(|_| "quoted static value")?;
    let links: Vec<_> = document
        .links()
        .iter()
        .filter(|link| link.authored == span)
        .collect();
    let [key_link, value_link] = links.as_slice() else {
        return Err("exact key and value links");
    };
    for (link, expected) in [(key_link, key.as_str()), (value_link, value.as_str())] {
        check(link.name.is_none() && link.segment)?;
        equal(
            document
                .as_str()
                .get(link.generated.start as usize..link.generated.end as usize),
            Some(expected),
        )?;
    }
    check(key_link.generated.end < value_link.generated.start)?;
    Ok(())
}
