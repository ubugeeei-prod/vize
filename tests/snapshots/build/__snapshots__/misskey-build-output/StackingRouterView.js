import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, Suspense as _Suspense, TransitionGroup as _TransitionGroup, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, resolveDynamicComponent as _resolveDynamicComponent, renderList as _renderList, mergeProps as _mergeProps, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, withCtx as _withCtx, unref as _unref, withModifiers as _withModifiers } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("path", { d: "M23.997,-42C47.903,-23.457 47.997,12 47.997,12L-0.003,12L-0.003,-96C-0.003,-96 0.091,-60.543 23.997,-42Z", style: "fill:var(--MI_THEME-panel);" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-rectangle" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-x" })
import { inject, provide, shallowRef } from 'vue'
import type { Router } from '@/router.js'
import { prefer } from '@/preferences.js'
import MkLoadingPage from '@/pages/_loading_.vue'
import { DI } from '@/di.js'
import { deepEqual } from '@/utility/deep-equal.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'StackingRouterView',
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
const tabs = shallowRef([{
	fullPath: router.getCurrentFullPath(),
	routePath: router.current.route.path,
	component: 'component' in router.current.route ? router.current.route.component : MkLoadingPage,
	props: router.current.props,
}]);
function mount() {
	const currentTab = tabs.value[tabs.value.length - 1];
	tabs.value = [currentTab];
}
function back() {
	if (tabs.value.length <= 1) return; // transitionの関係でタブが1つの状態でbackが呼ばれることがある
	const prev = tabs.value[tabs.value.length - 2];
	tabs.value = [...tabs.value.slice(0, tabs.value.length - 1)];
	router?.replaceByPath(prev.fullPath);
}
router.useListener('change', ({ resolved }) => {
	const currentTab = tabs.value[tabs.value.length - 1];
	const routePath = resolved.route.path;
	if (resolved == null || 'redirect' in resolved.route) return;
	if (resolved.route.path === currentTab.routePath && deepEqual(resolved.props, currentTab.props)) return;
	const fullPath = router.getCurrentFullPath();
	if (tabs.value.some(tab => tab.routePath === routePath && deepEqual(resolved.props, tab.props))) {
		const newTabs = [] as typeof tabs.value;
		for (const tab of tabs.value) {
			newTabs.push(tab);
			if (tab.routePath === routePath && deepEqual(resolved.props, tab.props)) {
				break;
			}
		}
		tabs.value = newTabs;
		return;
	}
	tabs.value = tabs.value.length >= prefer.s.numberOfPageCache ? [
		...tabs.value.slice(1),
		{
			fullPath: fullPath,
			routePath,
			component: resolved.route.component,
			props: resolved.props,
		},
	] : [...tabs.value, {
		fullPath: fullPath,
		routePath,
		component: resolved.route.component,
		props: resolved.props,
	}];
});
router.useListener('replace', ({ fullPath }) => {
	const currentTab = tabs.value[tabs.value.length - 1];
	currentTab.fullPath = fullPath;
	tabs.value = [...tabs.value.slice(0, tabs.value.length - 1), currentTab];
});

return (_ctx: any,_cache: any) => {
  const _component_MkLoading = _resolveComponent("MkLoading")

  return (_openBlock(), _createBlock(_TransitionGroup, {
      enterActiveClass: _unref(prefer).s.animation ? _ctx.$style.transition_x_enterActive : '',
      leaveActiveClass: _unref(prefer).s.animation ? _ctx.$style.transition_x_leaveActive : '',
      enterFromClass: _unref(prefer).s.animation ? _ctx.$style.transition_x_enterFrom : '',
      leaveToClass: _unref(prefer).s.animation ? _ctx.$style.transition_x_leaveTo : '',
      moveClass: _unref(prefer).s.animation ? _ctx.$style.transition_x_move : '',
      duration: 200,
      tag: "div",
      class: _normalizeClass(_ctx.$style.tabs)
    }, {
      default: _withCtx(() => [
        (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(tabs.value, (tab, i) => {
          return (_openBlock(), _createElementBlock("div", {
            key: tab.fullPath,
            class: _normalizeClass(_ctx.$style.tab),
            style: _normalizeStyle({ '--i': i - 1 })
          }, [
            (i > 0)
              ? (_openBlock(), _createElementBlock("div", {
                key: 0,
                class: _normalizeClass(_ctx.$style.tabBg),
                onClick: _cache[0] || (_cache[0] = ($event: any) => (back()))
              }))
              : _createCommentVNode("v-if", true),
            _createElementVNode("div", {
              class: _normalizeClass(_ctx.$style.tabFg),
              onClick: _cache[1] || (_cache[1] = _withModifiers(($event: any) => (back()), ["stop"]))
            }, [
              (i > 0)
                ? (_openBlock(), _createElementBlock("div", {
                  key: 0,
                  class: _normalizeClass(_ctx.$style.tabMenu)
                }, [
                  _createElementVNode("svg", {
                    class: _normalizeClass(_ctx.$style.tabMenuShape),
                    viewBox: "0 0 24 16"
                  }, [
                    _createElementVNode("g", { transform: "matrix(2.04108e-17,-0.333333,0.222222,1.36072e-17,21.3333,15.9989)" }, [
                      _hoisted_1
                    ])
                  ]),
                  _createElementVNode("button", {
                    class: _normalizeClass(["_button", _ctx.$style.tabMenuButton]),
                    onClick: _withModifiers(mount, ["stop"])
                  }, [
                    _hoisted_2
                  ]),
                  _createElementVNode("button", {
                    class: _normalizeClass(["_button", _ctx.$style.tabMenuButton]),
                    onClick: _withModifiers(back, ["stop"])
                  }, [
                    _hoisted_3
                  ])
                ]))
                : _createCommentVNode("v-if", true),
              (i > 0)
                ? (_openBlock(), _createElementBlock("div", {
                  key: 0,
                  class: _normalizeClass(_ctx.$style.tabBorder)
                }))
                : _createCommentVNode("v-if", true),
              _createElementVNode("div", {
                class: _normalizeClass(["_pageContainer", _ctx.$style.tabContent]),
                onClick: _withModifiers(() => {}, ["stop"])
              }, [
                _createVNode(_Suspense, { timeout: 0 }, {
                  fallback: _withCtx(() => [
                    _createVNode(_component_MkLoading)
                  ]),
                  default: _withCtx(() => [
                    _createVNode(_resolveDynamicComponent(tab.component), _mergeProps(Object.fromEntries(tab.props), {  }), null, 16 /* FULL_PROPS */)
                  ]),
                  _: 2 /* DYNAMIC */
                }, 8 /* PROPS */, ["timeout"])
              ], 8 /* PROPS */, ["onClick"])
            ])
          ], 4 /* STYLE */))
        }), 128 /* KEYED_FRAGMENT */))
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["enterActiveClass", "leaveActiveClass", "enterFromClass", "leaveToClass", "moveClass", "duration"]))
}
}

})
