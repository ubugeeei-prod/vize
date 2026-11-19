use vize_atelier_core::RuntimeHelper;
use vize_l4::runtime::VUE_SERVER_RENDERER;

use super::legacy_helper;

/// Every legacy helper, in declaration order.
const ALL: &[RuntimeHelper] = &[
    RuntimeHelper::Fragment,
    RuntimeHelper::Teleport,
    RuntimeHelper::Suspense,
    RuntimeHelper::KeepAlive,
    RuntimeHelper::BaseTransition,
    RuntimeHelper::Transition,
    RuntimeHelper::TransitionGroup,
    RuntimeHelper::OpenBlock,
    RuntimeHelper::CreateBlock,
    RuntimeHelper::CreateElementBlock,
    RuntimeHelper::CreateVNode,
    RuntimeHelper::CreateElementVNode,
    RuntimeHelper::CreateComment,
    RuntimeHelper::CreateText,
    RuntimeHelper::CreateStatic,
    RuntimeHelper::ResolveComponent,
    RuntimeHelper::ResolveDynamicComponent,
    RuntimeHelper::ResolveDirective,
    RuntimeHelper::ResolveFilter,
    RuntimeHelper::WithDirectives,
    RuntimeHelper::RenderList,
    RuntimeHelper::RenderSlot,
    RuntimeHelper::CreateSlots,
    RuntimeHelper::ToDisplayString,
    RuntimeHelper::MergeProps,
    RuntimeHelper::NormalizeClass,
    RuntimeHelper::NormalizeStyle,
    RuntimeHelper::NormalizeProps,
    RuntimeHelper::GuardReactiveProps,
    RuntimeHelper::ToHandlers,
    RuntimeHelper::Camelize,
    RuntimeHelper::Capitalize,
    RuntimeHelper::ToHandlerKey,
    RuntimeHelper::SetBlockTracking,
    RuntimeHelper::PushScopeId,
    RuntimeHelper::PopScopeId,
    RuntimeHelper::WithCtx,
    RuntimeHelper::Unref,
    RuntimeHelper::IsRef,
    RuntimeHelper::WithMemo,
    RuntimeHelper::IsMemoSame,
    RuntimeHelper::VShow,
    RuntimeHelper::VModelText,
    RuntimeHelper::VModelCheckbox,
    RuntimeHelper::VModelRadio,
    RuntimeHelper::VModelSelect,
    RuntimeHelper::VModelDynamic,
    RuntimeHelper::WithModifiers,
    RuntimeHelper::WithKeys,
    RuntimeHelper::SsrInterpolate,
    RuntimeHelper::SsrRenderVNode,
    RuntimeHelper::SsrRenderComponent,
    RuntimeHelper::SsrRenderSlot,
    RuntimeHelper::SsrRenderSlotInner,
    RuntimeHelper::SsrRenderAttrs,
    RuntimeHelper::SsrRenderAttr,
    RuntimeHelper::SsrRenderDynamicAttr,
    RuntimeHelper::SsrIncludeBooleanAttr,
    RuntimeHelper::SsrRenderClass,
    RuntimeHelper::SsrRenderStyle,
    RuntimeHelper::SsrRenderDynamicModel,
    RuntimeHelper::SsrGetDynamicModelProps,
    RuntimeHelper::SsrRenderList,
    RuntimeHelper::SsrLooseEqual,
    RuntimeHelper::SsrLooseContain,
    RuntimeHelper::SsrGetDirectiveProps,
    RuntimeHelper::SsrRenderTeleport,
    RuntimeHelper::SsrRenderSuspense,
];

#[test]
fn every_legacy_helper_maps_to_its_server_renderer_export() {
    assert_eq!(ALL.len(), RuntimeHelper::SsrRenderSuspense as usize + 1);
    for (position, &helper) in ALL.iter().enumerate() {
        assert_eq!(helper as usize, position, "{helper:?}");
        let mapped = legacy_helper(helper).and_then(|helper| VUE_SERVER_RENDERER.export(helper));
        if helper == RuntimeHelper::ResolveFilter {
            assert_eq!(mapped, None);
            continue;
        }
        let module = if helper.is_ssr() {
            "@vue/server-renderer"
        } else {
            "vue"
        };
        assert_eq!(mapped, Some((module, helper.name())), "{helper:?}");
    }
}
