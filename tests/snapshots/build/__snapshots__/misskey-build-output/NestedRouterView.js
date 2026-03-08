import { defineComponent as _defineComponent } from 'vue'
import { Suspense as _Suspense, openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, resolveComponent as _resolveComponent, resolveDynamicComponent as _resolveDynamicComponent, mergeProps as _mergeProps, withCtx as _withCtx } from "vue"

import { inject, provide, ref, shallowRef } from 'vue'
import type { Router } from '@/router.js'
import type { PathResolvedResult } from '@/lib/nirax.js'
import MkLoadingPage from '@/pages/_loading_.vue'
import { DI } from '@/di.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'NestedRouterView',
  props: {
    router: { type: null, required: false }
  },
  setup(__props: any) {

const props = __props
const router = props.router ?? inject(DI.router);
if (router == null) {
	throw new Error('no router provided');
}
const currentDepth = inject(DI.routerCurrentDepth, 0);
provide(DI.routerCurrentDepth, currentDepth + 1);
function resolveNested(current: PathResolvedResult, d = 0): PathResolvedResult | null {
	if (d === currentDepth) {
		return current;
	} else {
		if (current.child) {
			return resolveNested(current.child, d + 1);
		} else {
			return null;
		}
	}
}
const current = resolveNested(router.current)!;
const currentPageComponent = shallowRef('component' in current.route ? current.route.component : MkLoadingPage);
const currentPageProps = ref(current.props);
const key = ref(router.getCurrentFullPath());
router.useListener('change', ({ resolved }) => {
	const current = resolveNested(resolved);
	if (current == null || 'redirect' in current.route) return;
	currentPageComponent.value = current.route.component;
	currentPageProps.value = current.props;
	key.value = router.getCurrentFullPath();
});

return (_ctx: any,_cache: any) => {
  const _component_MkLoading = _resolveComponent("MkLoading")

  return (_openBlock(), _createBlock(_Suspense, { timeout: 0 }, {
      fallback: _withCtx(() => [
        _createVNode(_component_MkLoading)
      ]),
      default: _withCtx(() => [
        _createVNode(_resolveDynamicComponent(currentPageComponent.value), _mergeProps(Object.fromEntries(currentPageProps.value), {
          key: key.value
        }), null, 16 /* FULL_PROPS */)
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["timeout"]))
}
}

})
