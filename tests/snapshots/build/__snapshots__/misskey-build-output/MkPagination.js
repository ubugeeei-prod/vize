import { useSlots as _useSlots } from 'vue'
import { defineComponent as _defineComponent } from 'vue'
import { Transition as _Transition, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, resolveDynamicComponent as _resolveDynamicComponent, resolveDirective as _resolveDirective, renderSlot as _renderSlot, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref, withModifiers as _withModifiers } from "vue"

import { isLink } from '@@/js/is-link.js'
import { onMounted, computed, watch, unref } from 'vue'
import type { UnwrapRef } from 'vue'
import type { IPaginator } from '@/utility/paginator.js'
import MkButton from '@/components/MkButton.vue'
import { i18n } from '@/i18n.js'
import { prefer } from '@/preferences.js'
import MkPullToRefresh from '@/components/MkPullToRefresh.vue'
import MkPaginationControl from '@/components/MkPaginationControl.vue'
import * as os from '@/os.js'

export type MkPaginationOptions = {
	autoLoad?: boolean;
	/**
	 * ページネーションを進める方向
	 * - up: 上方向
	 * - down: 下方向 (default)
	 * - both: 双方向
	 *
	 * NOTE: この方向はページネーションの方向であって、アイテムの並び順ではない
	 */
	direction?: 'up' | 'down' | 'both';
	pullToRefresh?: boolean;
	withControl?: boolean;
	forceDisableInfiniteScroll?: boolean;
};

export default /*@__PURE__*/_defineComponent({
  __name: 'MkPagination',
  props: {
    autoLoad: { type: Boolean, required: false, default: true },
    direction: { type: String, required: false, default: 'down' },
    pullToRefresh: { type: Boolean, required: false, default: true },
    withControl: { type: Boolean, required: false, default: false },
    forceDisableInfiniteScroll: { type: Boolean, required: false, default: false },
    paginator: { type: null, required: true }
  },
  setup(__props: any) {

const props = __props
const shouldEnableInfiniteScroll = computed(() => {
	return prefer.r.enableInfiniteScroll.value && !props.forceDisableInfiniteScroll;
});
function onContextmenu(ev: PointerEvent) {
	if (ev.target && isLink(ev.target as HTMLElement)) return;
	if (window.getSelection()?.toString() !== '') return;
	// TODO: 並び順設定
	os.contextMenu([{
		icon: 'ti ti-refresh',
		text: i18n.ts.reload,
		action: () => {
			props.paginator.reload();
		},
	}], ev);
}
function getValue(v: IPaginator['items']) {
	return unref(v) as UnwrapRef<T['items']>;
}
if (props.autoLoad) {
	onMounted(() => {
		props.paginator.init();
	});
}
if (props.paginator.computedParams) {
	watch(props.paginator.computedParams, () => {
		props.paginator.reload();
	}, { immediate: false, deep: true });
}
const upButtonVisible = computed(() => {
	return props.paginator.order.value === 'oldest' ? props.paginator.canFetchOlder.value : props.paginator.canFetchNewer.value;
});
const upButtonLoading = computed(() => {
	return props.paginator.order.value === 'oldest' ? props.paginator.fetchingOlder.value : props.paginator.fetchingNewer.value;
});
function upButtonClick() {
	if (props.paginator.order.value === 'oldest') {
		props.paginator.fetchOlder();
	} else {
		props.paginator.fetchNewer();
	}
}
const downButtonVisible = computed(() => {
	return props.paginator.order.value === 'oldest' ? props.paginator.canFetchNewer.value : props.paginator.canFetchOlder.value;
});
const downButtonLoading = computed(() => {
	return props.paginator.order.value === 'oldest' ? props.paginator.fetchingNewer.value : props.paginator.fetchingOlder.value;
});
function downButtonClick() {
	if (props.paginator.order.value === 'oldest') {
		props.paginator.fetchNewer();
	} else {
		props.paginator.fetchOlder();
	}
}

return (_ctx: any,_cache: any) => {
  const _component_MkLoading = _resolveComponent("MkLoading")
  const _component_MkError = _resolveComponent("MkError")
  const _component_MkResult = _resolveComponent("MkResult")
  const _directive_appear = _resolveDirective("appear")

  return (_openBlock(), _createBlock(_resolveDynamicComponent(_unref(prefer).s.enablePullToRefresh && _ctx.pullToRefresh ? MkPullToRefresh : 'div'), {
      refresher: () => _ctx.paginator.reload(),
      onContextmenu: _withModifiers(onContextmenu, ["prevent","stop"])
    }, {
      default: _withCtx(() => [
        _createElementVNode("div", null, [
          (props.withControl)
            ? (_openBlock(), _createBlock(MkPaginationControl, {
              key: 0,
              paginator: _ctx.paginator,
              style: "margin-bottom: 10px"
            }, null, 8 /* PROPS */, ["paginator"]))
            : _createCommentVNode("v-if", true),
          _createTextVNode("\n\n\t\t" + "\n\t\t"),
          _createVNode(_Transition, {
            enterActiveClass: _unref(prefer).s.animation ? _ctx.$style.transition_fade_enterActive : '',
            leaveActiveClass: _unref(prefer).s.animation ? _ctx.$style.transition_fade_leaveActive : '',
            enterFromClass: _unref(prefer).s.animation ? _ctx.$style.transition_fade_enterFrom : '',
            leaveToClass: _unref(prefer).s.animation ? _ctx.$style.transition_fade_leaveTo : '',
            mode: _unref(prefer).s.animation ? 'out-in' : undefined
          }, {
            default: _withCtx(() => [
              (_ctx.paginator.fetching.value)
                ? (_openBlock(), _createBlock(_component_MkLoading, { key: 0 }))
                : (_ctx.paginator.error.value)
                  ? (_openBlock(), _createBlock(_component_MkError, {
                    key: 1,
                    onRetry: _cache[0] || (_cache[0] = ($event: any) => (_ctx.paginator.init()))
                  }))
                : (_ctx.paginator.items.value.length === 0)
                  ? (_openBlock(), _createElementBlock("div", { key: "_empty_" }, [
                    _renderSlot(_ctx.$slots, "empty", {}, () => [
                      _createVNode(_component_MkResult, { type: "empty" })
                    ])
                  ]))
                : (_openBlock(), _createElementBlock("div", {
                  key: "_root_",
                  class: "_gaps"
                }, [
                  (_ctx.direction === 'up' || _ctx.direction === 'both')
                    ? (_openBlock(), _createElementBlock("div", { key: 0 }, [
                      (!upButtonLoading.value)
                        ? (_openBlock(), _createBlock(MkButton, {
                          key: 0,
                          class: _normalizeClass(_ctx.$style.more),
                          primary: "",
                          rounded: "",
                          onClick: upButtonClick
                        }, {
                          default: _withCtx(() => [
                            _createTextVNode(_toDisplayString(_unref(i18n).ts.loadMore), 1 /* TEXT */)
                          ]),
                          _: 1 /* STABLE */
                        }))
                        : (_openBlock(), _createBlock(_component_MkLoading, { key: 1 }))
                    ]))
                    : _createCommentVNode("v-if", true),
                  _renderSlot(_ctx.$slots, "default", {
                    items: getValue(_ctx.paginator.items),
                    fetching: _ctx.paginator.fetching.value || _ctx.paginator.fetchingOlder.value
                  }),
                  (_ctx.direction === 'down' || _ctx.direction === 'both')
                    ? (_openBlock(), _createElementBlock("div", { key: 0 }, [
                      (!downButtonLoading.value)
                        ? (_openBlock(), _createBlock(MkButton, {
                          key: 0,
                          class: _normalizeClass(_ctx.$style.more),
                          primary: "",
                          rounded: "",
                          onClick: downButtonClick
                        }, {
                          default: _withCtx(() => [
                            _createTextVNode(_toDisplayString(_unref(i18n).ts.loadMore), 1 /* TEXT */)
                          ]),
                          _: 1 /* STABLE */
                        }))
                        : (_openBlock(), _createBlock(_component_MkLoading, { key: 1 }))
                    ]))
                    : _createCommentVNode("v-if", true)
                ]))
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["enterActiveClass", "leaveActiveClass", "enterFromClass", "leaveToClass", "mode"])
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["refresher"]))
}
}

})
