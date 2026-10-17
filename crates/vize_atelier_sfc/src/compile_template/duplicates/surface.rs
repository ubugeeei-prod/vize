//! Consume the existing DOM tree without a second parse.
//! The ordinary generic DOM surface has the same neutral v-pre lexical policy
//! as the existing duplicate callback. A dialect-frozen tree is not substituted.

use super::{Candidate, SourceLocation};
use vize_l0::SourceRoot;
use vize_l1::{
    SurfaceChild, SurfaceError, SurfaceTree,
    markup::{ArgSyntax, DirectivePrefix, DirectiveSyntax, VueDirectives},
};

/// None retains the whole existing callback result; partial spans never escape.
pub(super) fn find_in_surface(
    tree: &SurfaceTree<'_>,
    errors: &[SurfaceError],
) -> Option<Vec<SourceLocation>> {
    if !errors.is_empty() {
        return None;
    }
    let block = SourceRoot::new(tree.source).ok()?.whole_block();
    let mut frames = vize_l0::SmallVec::<[_; 8]>::new();
    frames.push(tree.children.iter());
    let mut seen = Vec::new();
    let mut duplicates = Vec::new();
    while let Some(children) = frames.last_mut() {
        let Some(child) = children.next() else {
            frames.pop();
            continue;
        };
        let element = match child {
            SurfaceChild::Element(element) => element,
            SurfaceChild::Unexpected(_) => return None,
            _ => continue,
        };
        // NamespaceSink treats SVG title children as markup; the retained
        // duplicate callback treats every title body as raw text. Decline the
        // complete observation rather than consume different lexical facts.
        if element.tag().eq_ignore_ascii_case("title")
            || element.open.gt.is_missing()
            || element.open.is_verbatim()
        {
            return None;
        }
        seen.clear();
        for attribute in &element.open.attrs {
            if attribute.name.is_missing() {
                return None;
            }
            let span = block.span_of(attribute.name.text)?;
            let raw = attribute.name.text;
            let directive = VueDirectives.decompose(raw, span.start).ok()?;
            let (name, argument, modifiers) = if let Some(directive) = directive {
                let end = if directive.prefix == DirectivePrefix::Full {
                    directive.name.end
                } else {
                    span.start.checked_add(1)?
                };
                let name = tree.source.get(span.start as usize..end as usize)?;
                // The shared lexer has a separate recovery event for a bracket
                // in a full head without ':'. Keep that callback route intact.
                if name.contains('[') || name.contains(']') {
                    return None;
                }
                let argument = match directive.arg {
                    Some(ArgSyntax::Static(span)) => Some((
                        tree.source.get(span.start as usize..span.end as usize)?,
                        false,
                    )),
                    Some(ArgSyntax::Dynamic(span)) => Some((
                        tree.source.get(span.start as usize..span.end as usize)?,
                        true,
                    )),
                    None => None,
                };
                let modifiers = tree
                    .source
                    .get(directive.modifiers.start as usize..directive.modifiers.end as usize)?;
                let modifiers = modifiers
                    .strip_prefix('.')
                    .map_or_else(Vec::new, |run| run.split('.').collect());
                (name, argument, modifiers)
            } else {
                (raw, None, Vec::new())
            };
            let candidate = Candidate {
                name,
                argument,
                modifiers,
                start: span.start as usize,
                end: span.end as usize,
                directive: directive.is_some(),
            };
            if candidate.may_repeat() {
                continue;
            }
            if seen.iter().any(|prior| candidate.same_name(prior)) {
                duplicates.push(SourceLocation::new(span.start, span.end));
            }
            seen.push(candidate);
        }
        frames.push(element.children.iter());
    }
    // HTML recovery may reparent nodes. Diagnostics keep authored byte order.
    duplicates.sort_by_key(|loc| loc.span.start);
    Some(duplicates)
}

#[cfg(test)]
#[test]
fn existing_native_tree_matches_complete_callback_spans_for_original_headers() {
    for source in [
        r#"<p :one="a" :two="b" @click="a" @click="b" />"#,
        r#"<p v-if="a" v-if="b" />"#,
        r#"<p CLASS="a" class="b" />"#,
        r#"<p V-IF="a" v-if="b" />"#,
        r#"<p :key..prop="a" :key.prop.="b" />"#,
        r#"<p v-bind="a" v-bind="b" :one="a" v-bind:one="b" />"#,
        r#"<p :[key].prop="a" :[key].prop="b" />"#,
        r#"<p :key.a.b="a" :key.b.a="b" :key.a.b="c" />"#,
        r#"<p v-pre :key="a" :key="b"><i class="x" CLASS="y" /></p>"#,
        r#"<p title="<i id=x id=x>"><!-- <b id=x id=x> --></p>"#,
        "<p title='雪🌸' id='a' id='b'><i v-if='x' v-if='y'/></p>",
    ] {
        let arena = vize_l0::Allocator::default();
        let (tree, errors) = vize_l1::parse(&arena, source);
        let actual =
            find_in_surface(&tree, &errors).expect("declared original neutral-surface controls");
        assert_eq!(
            serde_json::to_value(actual).unwrap(),
            serde_json::to_value(super::find_duplicate_props(source)).unwrap(),
            "{source}"
        );
    }
}

#[cfg(test)]
#[test]
fn unsupported_surface_keeps_the_complete_original_callback_route() {
    for source in [
        "<p title='unterminated",
        "<p v-if[a]='x' v-if[a]='y'/>",
        "<div v-pre>{{</div>",
    ] {
        let arena = vize_l0::Allocator::default();
        let (tree, errors) = vize_l1::parse(&arena, source);
        assert!(find_in_surface(&tree, &errors).is_none(), "{source}");
    }
}

#[cfg(test)]
#[test]
fn dialect_frozen_surface_is_not_substituted_for_the_neutral_dom_tree() {
    let source = "<div v-pre :key='a' :key='b'></div>";
    let arena = vize_l0::Allocator::default();
    let frozen = vize_l1::markup::parse_component(&arena, source).unwrap();
    assert!(find_in_surface(&frozen.tree, &frozen.errors).is_none());
    let (neutral, errors) = vize_l1::parse(&arena, source);
    let actual = find_in_surface(&neutral, &errors).unwrap();
    assert_eq!(
        serde_json::to_value(actual).unwrap(),
        serde_json::to_value(super::find_duplicate_props(source)).unwrap()
    );
}

#[cfg(test)]
#[test]
fn every_title_observation_declines_without_retaining_partial_spans() {
    for source in [
        "<title><g id='a' id='b'></g></title>",
        "<svg><title><g id='a' id='b'></g></title></svg>",
        "<svg><TITLE><g id='a' id='b'></g></TITLE></svg>",
        "<svg><foreignObject><title><g id='a' id='b'></g></title></foreignObject></svg>",
        "<div id='a' id='b'><svg><title><g id='x' id='y'></g></title></svg></div>",
    ] {
        let arena = vize_l0::Allocator::default();
        let (tree, errors) = vize_l1::parse(&arena, source);
        assert!(find_in_surface(&tree, &errors).is_none(), "{source}");
    }
}
