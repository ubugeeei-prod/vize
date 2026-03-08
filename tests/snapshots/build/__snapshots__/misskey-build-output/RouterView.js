import { defineComponent as _defineComponent } from 'vue'
import { Suspense as _Suspense, KeepAlive as _KeepAlive, openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, resolveComponent as _resolveComponent, resolveDynamicComponent as _resolveDynamicComponent, mergeProps as _mergeProps, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"

import { inject, nextTick, onMounted, provide, ref, shallowRef, useTemplateRef } from 'vue'
import type { Router } from '@/router.js'
import { prefer } from '@/preferences.js'
import MkLoadingPage from '@/pages/_loading_.vue'
import { DI } from '@/di.js'
import { randomId } from '@/utility/random-id.js'
import { deepEqual } from '@/utility/deep-equal.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'RouterView',
  props: {
    router: { type: null, required: false }
  },
  setup(__props: any) {

const props = __props
const _router = props.router ?? inject(DI.router);
if (_router == null) {
	throw new Error('no router provided');
}
const router = _router;
const viewId = randomId();
provide(DI.viewId, viewId);
const currentDepth = inject(DI.routerCurrentDepth, 0);
provide(DI.routerCurrentDepth, currentDepth + 1);
const current = router.current;
const currentPageComponent = shallowRef('component' in current.route ? current.route.component : MkLoadingPage);
const currentPageProps = ref(current.props);
let currentRoutePath = current.route.path;
const key = ref(router.getCurrentFullPath());
router.useListener('change', ({ resolved }) => {
	if (resolved == null || 'redirect' in resolved.route) return;
	if (resolved.route.path === currentRoutePath && deepEqual(resolved.props, currentPageProps.value)) return;
	currentPageComponent.value = resolved.route.component;
	currentPageProps.value = resolved.props;
	key.value = router.getCurrentFullPath();
	currentRoutePath = resolved.route.path;
});

return (_ctx: any,_cache: any) => {
  const _component_MkLoading = _resolveComponent("MkLoading")

  return (_openBlock(), _createElementBlock("div", {
      class: _normalizeClass(["_pageContainer", _ctx.$style.root])
    }, [ _createVNode(_KeepAlive, { max: _unref(prefer).s.numberOfPageCache }, [ _createVNode(_Suspense, { timeout: 0 }, {
          fallback: _withCtx(() => [
            _createVNode(_component_MkLoading)
          ]),
          default: _withCtx(() => [
            _createVNode(_resolveDynamicComponent(currentPageComponent.value), _mergeProps(Object.fromEntries(currentPageProps.value), {
              key: key.value
            }), null, 16 /* FULL_PROPS */)
          ]),
          _: 1 /* STABLE */
        }, 8 /* PROPS */, ["timeout"]) ], 1032 /* PROPS, DYNAMIC_SLOTS */, ["max"]) ]))
}
}

})
