//! Append genuine roots directly into the caller's existing Writer.

use super::{VaporError, VaporErrorKind, error, literal, number};
use crate::runtime::{Runtime, vocabulary};
use crate::write::{LinkSink, Writer};
use vize_l3::decision::vapor::{VaporFacts, VaporPart};

pub(super) fn static_roots<L: LinkSink, const SETUP: bool>(
    prelude: &mut Writer<L>,
    render: &mut Writer<L>,
    facts: &VaporFacts<'_, '_>,
) -> Result<(), VaporError> {
    let runtime = vocabulary(Runtime::VueVapor);
    for (index, root) in facts.roots().iter().enumerate() {
        let parts = facts
            .parts(root)
            .ok_or_else(|| error(VaporErrorKind::ForeignRoot))?;
        render.newline();
        render.push("const n");
        number(render, index);
        render.push(" = ");
        let helper = runtime
            .helper("template")
            .ok_or_else(|| error(VaporErrorKind::RuntimeHelper))?;
        prelude.use_helper(helper);
        prelude.push("const t");
        number(prelude, index);
        prelude.push(" = _template(\"");
        if let [VaporPart::Text { text, .. }] = parts {
            prelude.anchor(text.span.start);
            // Actual template() uses a raw Text branch when the first byte
            // is not '<'. L3 refuses that markup-shaped root spelling.
            literal::raw(prelude, text.content);
        } else {
            for part in parts {
                // Actual stock inline setup escapes HTML comment contents.
                // Real browsers retain those entities in Comment.data; the
                // established standalone/scriptless byte contract stays separate.
                if SETUP
                    && let VaporPart::Comment { node, comment } = part
                    && comment.content.contains(['&', '<', '>', '"', '\''])
                {
                    return Err(VaporError {
                        node: Some(*node),
                        span: comment.span,
                        kind: VaporErrorKind::CommentNormalization,
                    });
                }
                literal::html_part(prelude, part);
            }
        }
        // Actual rc.9 TemplateFlags.STATIC=2 and ROOT=1. L3 decides
        // root fallthrough; the encoder owns the pinned numeric spelling.
        prelude.push(if facts.inherit_attrs() == Some(root.node()) {
            "\", 3)\n"
        } else {
            "\", 2)\n"
        });
        render.push("t");
        number(render, index);
        render.push("()");
    }
    Ok(())
}

pub(super) fn return_roots<L: LinkSink>(render: &mut Writer<L>, count: usize) {
    render.newline();
    render.push("return ");
    if count != 1 {
        render.push("[");
    }
    for index in 0..count {
        if index != 0 {
            render.push(", ");
        }
        render.push("n");
        number(render, index);
    }
    if count != 1 {
        render.push("]");
    }
}
