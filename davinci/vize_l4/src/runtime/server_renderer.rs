//! Named helpers of the server-rendering vocabulary
//! ([`VUE_SERVER_RENDERER`](super::VUE_SERVER_RENDERER)).
//!
//! Each constant is the export's index, resolved from the vocabulary table
//! while compiling, so an SSR emitter records a helper with one mask test and
//! never compares names. A name missing from the table fails the build.

use super::Helper;
use super::vue::{DOM_NAMES, SERVER_NAMES};

const fn same(left: &str, right: &str) -> bool {
    let (mut left, mut right) = (left.as_bytes(), right.as_bytes());
    while let ([a, left_rest @ ..], [b, right_rest @ ..]) = (left, right) {
        if *a != *b {
            return false;
        }
        (left, right) = (left_rest, right_rest);
    }
    left.is_empty() && right.is_empty()
}

const fn position(mut names: &[&str], name: &str) -> Option<usize> {
    let mut index = 0;
    while let [first, rest @ ..] = names {
        if same(first, name) {
            return Some(index);
        }
        names = rest;
        index += 1;
    }
    None
}

/// The vocabulary index of `name`: server-renderer exports first, then the
/// shared Vue exports. [`Helper::LIMIT`] when the vocabulary lacks it.
const fn index(name: &str) -> usize {
    match position(SERVER_NAMES, name) {
        Some(index) => index,
        None => match position(DOM_NAMES, name) {
            Some(index) => SERVER_NAMES.len() + index,
            None => Helper::LIMIT,
        },
    }
}

macro_rules! helpers {
    ($($konst:ident = $name:literal,)*) => {
        $(
            #[doc = concat!("`", $name, "`")]
            pub const $konst: Helper = {
                let index = index($name);
                assert!(index < Helper::LIMIT, concat!("not a server-renderer export: ", $name));
                Helper(index as u8)
            };
        )*

        /// Every named helper with its export name, for table tests.
        #[cfg(test)]
        pub(super) const NAMED: &[(Helper, &str)] = &[$(($konst, $name)),*];
    };
}

helpers! {
    SSR_INTERPOLATE = "ssrInterpolate",
    SSR_RENDER_VNODE = "ssrRenderVNode",
    SSR_RENDER_COMPONENT = "ssrRenderComponent",
    SSR_RENDER_SLOT = "ssrRenderSlot",
    SSR_RENDER_ATTRS = "ssrRenderAttrs",
    SSR_RENDER_ATTR = "ssrRenderAttr",
    SSR_INCLUDE_BOOLEAN_ATTR = "ssrIncludeBooleanAttr",
    SSR_RENDER_CLASS = "ssrRenderClass",
    SSR_RENDER_STYLE = "ssrRenderStyle",
    SSR_RENDER_DYNAMIC_MODEL = "ssrRenderDynamicModel",
    SSR_GET_DYNAMIC_MODEL_PROPS = "ssrGetDynamicModelProps",
    SSR_RENDER_LIST = "ssrRenderList",
    SSR_LOOSE_EQUAL = "ssrLooseEqual",
    SSR_LOOSE_CONTAIN = "ssrLooseContain",
    SSR_GET_DIRECTIVE_PROPS = "ssrGetDirectiveProps",
    SSR_RENDER_TELEPORT = "ssrRenderTeleport",
    SSR_RENDER_SUSPENSE = "ssrRenderSuspense",
    FRAGMENT = "Fragment",
    OPEN_BLOCK = "openBlock",
    CREATE_BLOCK = "createBlock",
    CREATE_VNODE = "createVNode",
    CREATE_ELEMENT_VNODE = "createElementVNode",
    CREATE_COMMENT_VNODE = "createCommentVNode",
    CREATE_TEXT_VNODE = "createTextVNode",
    RESOLVE_DYNAMIC_COMPONENT = "resolveDynamicComponent",
    RENDER_LIST = "renderList",
    RENDER_SLOT = "renderSlot",
    CREATE_SLOTS = "createSlots",
    TO_DISPLAY_STRING = "toDisplayString",
    MERGE_PROPS = "mergeProps",
    NORMALIZE_PROPS = "normalizeProps",
    GUARD_REACTIVE_PROPS = "guardReactiveProps",
    TO_HANDLERS = "toHandlers",
    CAMELIZE = "camelize",
    TO_HANDLER_KEY = "toHandlerKey",
    WITH_CTX = "withCtx",
}
