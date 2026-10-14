//! Component prop keys for `v-bind` modifiers.

use super::{SimpleExpressionNode, String, TransformContext};

/// The prop key a `v-bind` modifier asks for, as Vue's compiler spells it for
/// a component: `.camel` camelizes a static key, `.prop` reads `.key` and
/// `.attr` reads `^key`. The runtime sets a `.` key as a DOM property and a
/// `^` key as an attribute when the props fall through to an element.
pub(super) fn modified_prop_key<'a>(
    ctx: &TransformContext<'a>,
    key: &SimpleExpressionNode<'a>,
    modifiers: &[SimpleExpressionNode<'a>],
) -> SimpleExpressionNode<'a> {
    let (mut camel, mut prefix) = (false, None);
    for modifier in modifiers {
        match modifier.content {
            "camel" => camel = true,
            "prop" => prefix = Some('.'),
            "attr" => prefix = Some('^'),
            _ => {}
        }
    }
    if !camel && prefix.is_none() {
        return SimpleExpressionNode::from_node(key);
    }
    if key.is_static {
        let mut content = String::default();
        content.extend(prefix);
        if camel {
            content.push_str(&vize_l0::general::camelize(key.content));
        } else {
            content.push_str(key.content);
        }
        return SimpleExpressionNode::new(ctx.allocator.alloc_str(&content), true, key.loc.clone());
    }
    // A dynamic key is prefixed at run time; camelizing one needs a runtime
    // helper and stays as authored.
    let Some(prefix) = prefix else {
        return SimpleExpressionNode::from_node(key);
    };
    let mut content = String::default();
    content.push('"');
    content.push(prefix);
    content.push_str("\" + (");
    content.push_str(key.content);
    content.push_str(" || \"\")");
    SimpleExpressionNode::new(ctx.allocator.alloc_str(&content), false, key.loc.clone())
}
