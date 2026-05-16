import { Transition as _Transition, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, resolveDynamicComponent as _resolveDynamicComponent, resolveDirective as _resolveDirective, withDirectives as _withDirectives, renderSlot as _renderSlot, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref, vShow as _vShow, withModifiers as _withModifiers } from "vue";
import { isLink } from "@@/js/is-link.js";
import { onMounted, computed, watch, unref } from "vue";
import MkButton from "@/components/MkButton.vue";
import { i18n } from "@/i18n.js";
import { prefer } from "@/preferences.js";
import MkPullToRefresh from "@/components/MkPullToRefresh.vue";
import MkPaginationControl from "@/components/MkPaginationControl.vue";
import * as os from "@/os.js";
export {};
export default {
  __name: "MkPagination",
  props: {
    autoLoad: {
      type: Boolean,
      required: false,
      default: true
    },
    direction: {
      type: String,
      required: false,
      default: "down"
    },
    pullToRefresh: {
      type: Boolean,
      required: false,
      default: true
    },
    withControl: {
      type: Boolean,
      required: false,
      default: false
    },
    forceDisableInfiniteScroll: {
      type: Boolean,
      required: false,
      default: false
    },
    paginator: {
      type: null,
      required: true
    }
  },
  setup(__props) {
    const props = __props;
    const shouldEnableInfiniteScroll = computed(() => {
      return prefer.r.enableInfiniteScroll.value && !props.forceDisableInfiniteScroll;
    });
    function onContextmenu(ev) {
      if (ev.target && isLink(ev.target)) return;
      if (window.getSelection()?.toString() !== "") return;
      // TODO: 並び順設定
      os.contextMenu([{
        icon: "ti ti-refresh",
        text: i18n.ts.reload,
        action: () => {
          props.paginator.reload();
        }
      }], ev);
    }
    function getValue(v) {
      return unref(v);
    }
    if (props.autoLoad) {
      onMounted(() => {
        props.paginator.init();
      });
    }
    if (props.paginator.computedParams) {
      watch(props.paginator.computedParams, () => {
        props.paginator.reload();
      }, {
        immediate: false,
        deep: true
      });
    }
    const upButtonVisible = computed(() => {
      return props.paginator.order.value === "oldest" ? props.paginator.canFetchOlder.value : props.paginator.canFetchNewer.value;
    });
    const upButtonLoading = computed(() => {
      return props.paginator.order.value === "oldest" ? props.paginator.fetchingOlder.value : props.paginator.fetchingNewer.value;
    });
    function upButtonClick() {
      if (props.paginator.order.value === "oldest") {
        props.paginator.fetchOlder();
      } else {
        props.paginator.fetchNewer();
      }
    }
    const downButtonVisible = computed(() => {
      return props.paginator.order.value === "oldest" ? props.paginator.canFetchNewer.value : props.paginator.canFetchOlder.value;
    });
    const downButtonLoading = computed(() => {
      return props.paginator.order.value === "oldest" ? props.paginator.fetchingNewer.value : props.paginator.fetchingOlder.value;
    });
    function downButtonClick() {
      if (props.paginator.order.value === "oldest") {
        props.paginator.fetchNewer();
      } else {
        props.paginator.fetchOlder();
      }
    }
    return (_ctx, _cache) => {
      const _component_MkLoading = _resolveComponent("MkLoading");
      const _component_MkError = _resolveComponent("MkError");
      const _component_MkResult = _resolveComponent("MkResult");
      const _directive_appear = _resolveDirective("appear");
      return _openBlock(), _createBlock(_resolveDynamicComponent(_unref(prefer).s.enablePullToRefresh && __props.pullToRefresh ? MkPullToRefresh : "div"), {
        refresher: () => __props.paginator.reload(),
        onContextmenu: _withModifiers(onContextmenu, ["prevent", "stop"])
      }, {
        default: _withCtx(() => [_createElementVNode("div", null, [props.withControl ? (_openBlock(), _createBlock(MkPaginationControl, {
          key: 0,
          paginator: __props.paginator,
          style: "margin-bottom: 10px"
        }, null, 8, ["paginator"])) : _createCommentVNode("v-if", true), _createVNode(_Transition, {
          enterActiveClass: _unref(prefer).s.animation ? _ctx.$style.transition_fade_enterActive : "",
          leaveActiveClass: _unref(prefer).s.animation ? _ctx.$style.transition_fade_leaveActive : "",
          enterFromClass: _unref(prefer).s.animation ? _ctx.$style.transition_fade_enterFrom : "",
          leaveToClass: _unref(prefer).s.animation ? _ctx.$style.transition_fade_leaveTo : "",
          mode: _unref(prefer).s.animation ? "out-in" : undefined
        }, {
          default: _withCtx(() => [__props.paginator.fetching.value ? (_openBlock(), _createBlock(_component_MkLoading, { key: 0 })) : __props.paginator.error.value ? (_openBlock(), _createBlock(_component_MkError, {
            key: 1,
            onRetry: _cache[0] || (_cache[0] = ($event) => __props.paginator.init())
          })) : __props.paginator.items.value.length === 0 ? (_openBlock(), _createElementBlock("div", { key: "_empty_" }, [_renderSlot(_ctx.$slots, "empty", {}, () => [_createVNode(_component_MkResult, { type: "empty" })])])) : (_openBlock(), _createElementBlock("div", {
            key: "_root_",
            class: "_gaps"
          }, [
            __props.direction === "up" || __props.direction === "both" ? _withDirectives((_openBlock(), _createElementBlock(
              "div",
              { key: 0 },
              [!upButtonLoading.value ? _withDirectives((_openBlock(), _createBlock(
                MkButton,
                {
                  key: 0,
                  class: _normalizeClass(_ctx.$style.more),
                  primary: "",
                  rounded: "",
                  onClick: upButtonClick
                },
                {
                  default: _withCtx(() => [_createTextVNode(
                    _toDisplayString(_unref(i18n).ts.loadMore),
                    1
                    /* TEXT */
                  )]),
                  _: 1
                },
                2
                /* CLASS */
              )), [[_directive_appear, shouldEnableInfiniteScroll.value ? upButtonClick : null]]) : (_openBlock(), _createBlock(_component_MkLoading, { key: 1 }))],
              512
              /* NEED_PATCH */
            )), [[_vShow, upButtonVisible.value]]) : _createCommentVNode("v-if", true),
            _renderSlot(_ctx.$slots, "default", {
              items: getValue(__props.paginator.items),
              fetching: __props.paginator.fetching.value || __props.paginator.fetchingOlder.value
            }),
            __props.direction === "down" || __props.direction === "both" ? _withDirectives((_openBlock(), _createElementBlock(
              "div",
              { key: 0 },
              [!downButtonLoading.value ? _withDirectives((_openBlock(), _createBlock(
                MkButton,
                {
                  key: 0,
                  class: _normalizeClass(_ctx.$style.more),
                  primary: "",
                  rounded: "",
                  onClick: downButtonClick
                },
                {
                  default: _withCtx(() => [_createTextVNode(
                    _toDisplayString(_unref(i18n).ts.loadMore),
                    1
                    /* TEXT */
                  )]),
                  _: 1
                },
                2
                /* CLASS */
              )), [[_directive_appear, shouldEnableInfiniteScroll.value ? downButtonClick : null]]) : (_openBlock(), _createBlock(_component_MkLoading, { key: 1 }))],
              512
              /* NEED_PATCH */
            )), [[_vShow, downButtonVisible.value]]) : _createCommentVNode("v-if", true)
          ]))]),
          _: 3
        }, 1032, [
          "enterActiveClass",
          "leaveActiveClass",
          "enterFromClass",
          "leaveToClass",
          "mode"
        ])])]),
        _: 3
      }, 1032, ["refresher"]);
    };
  }
};
