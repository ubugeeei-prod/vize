import { Fragment as _Fragment, Transition as _Transition, openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, withDirectives as _withDirectives, renderSlot as _renderSlot, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref, vShow as _vShow } from "vue";
const _hoisted_1 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-chevron-up" });
const _hoisted_2 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-chevron-down" });
import { onMounted, onUnmounted, ref, useTemplateRef, watch } from "vue";
import { prefer } from "@/preferences.js";
import { i18n } from "@/i18n.js";
export default {
  __name: "MkContainer",
  props: {
    showHeader: {
      type: Boolean,
      required: false,
      default: true
    },
    thin: {
      type: Boolean,
      required: false
    },
    naked: {
      type: Boolean,
      required: false
    },
    foldable: {
      type: Boolean,
      required: false
    },
    scrollable: {
      type: Boolean,
      required: false
    },
    expanded: {
      type: Boolean,
      required: false,
      default: true
    },
    maxHeight: {
      type: [Number, null],
      required: false,
      default: null
    }
  },
  setup(__props) {
    const props = __props;
    const rootEl = useTemplateRef("rootEl");
    const contentEl = useTemplateRef("contentEl");
    const headerEl = useTemplateRef("headerEl");
    const showBody = ref(props.expanded);
    const ignoreOmit = ref(false);
    const omitted = ref(false);
    function enter(el) {
      if (!(el instanceof HTMLElement)) return;
      const elementHeight = el.getBoundingClientRect().height;
      el.style.height = "0";
      el.offsetHeight;
      el.style.height = `${Math.min(elementHeight, props.maxHeight ?? Infinity)}px`;
    }
    function afterEnter(el) {
      if (!(el instanceof HTMLElement)) return;
      el.style.height = "";
    }
    function leave(el) {
      if (!(el instanceof HTMLElement)) return;
      const elementHeight = el.getBoundingClientRect().height;
      el.style.height = `${elementHeight}px`;
      el.offsetHeight;
      el.style.height = "0";
    }
    function afterLeave(el) {
      if (!(el instanceof HTMLElement)) return;
      el.style.height = "";
    }
    const calcOmit = () => {
      if (omitted.value || ignoreOmit.value || props.maxHeight == null) return;
      if (!contentEl.value) return;
      const height = contentEl.value.offsetHeight;
      omitted.value = height > props.maxHeight;
    };
    const omitObserver = new ResizeObserver((entries, observer) => {
      calcOmit();
    });
    function showMore() {
      ignoreOmit.value = true;
      omitted.value = false;
    }
    onMounted(() => {
      watch(showBody, (v) => {
        if (!rootEl.value) return;
        const headerHeight = props.showHeader ? headerEl.value?.offsetHeight ?? 0 : 0;
        rootEl.value.style.minHeight = `${headerHeight}px`;
        if (v) {
          rootEl.value.style.flexBasis = "auto";
        } else {
          rootEl.value.style.flexBasis = `${headerHeight}px`;
        }
      }, { immediate: true });
      if (rootEl.value) rootEl.value.style.setProperty("--maxHeight", props.maxHeight + "px");
      calcOmit();
      if (contentEl.value) omitObserver.observe(contentEl.value);
    });
    onUnmounted(() => {
      omitObserver.disconnect();
    });
    return (_ctx, _cache) => {
      return _openBlock(), _createElementBlock(
        "div",
        {
          ref_key: "rootEl",
          ref: rootEl,
          class: _normalizeClass(["_panel", [_ctx.$style.root, {
            [_ctx.$style.naked]: __props.naked,
            [_ctx.$style.thin]: __props.thin,
            [_ctx.$style.scrollable]: __props.scrollable
          }]])
        },
        [__props.showHeader ? (_openBlock(), _createElementBlock(
          "header",
          {
            key: 0,
            ref_key: "headerEl",
            ref: headerEl,
            class: _normalizeClass(_ctx.$style.header)
          },
          [_createElementVNode(
            "div",
            { class: _normalizeClass(_ctx.$style.title) },
            [_createElementVNode(
              "span",
              { class: _normalizeClass(_ctx.$style.titleIcon) },
              [_renderSlot(_ctx.$slots, "icon")],
              2
              /* CLASS */
            ), _renderSlot(_ctx.$slots, "header")],
            2
            /* CLASS */
          ), _createElementVNode(
            "div",
            { class: _normalizeClass(_ctx.$style.headerSub) },
            [_renderSlot(_ctx.$slots, "func", { buttonStyleClass: _ctx.$style.headerButton }), __props.foldable ? (_openBlock(), _createElementBlock(
              "button",
              {
                key: 0,
                class: _normalizeClass(["_button", _ctx.$style.headerButton]),
                onClick: _cache[0] || (_cache[0] = () => showBody.value = !showBody.value)
              },
              [showBody.value ? (_openBlock(), _createElementBlock(
                _Fragment,
                { key: 0 },
                [_hoisted_1],
                64
                /* STABLE_FRAGMENT */
              )) : (_openBlock(), _createElementBlock(
                _Fragment,
                { key: 1 },
                [_hoisted_2],
                64
                /* STABLE_FRAGMENT */
              ))],
              2
              /* CLASS */
            )) : _createCommentVNode("v-if", true)],
            2
            /* CLASS */
          )],
          2
          /* CLASS */
        )) : _createCommentVNode("v-if", true), _createVNode(_Transition, {
          enterActiveClass: _unref(prefer).s.animation ? _ctx.$style.transition_toggle_enterActive : "",
          leaveActiveClass: _unref(prefer).s.animation ? _ctx.$style.transition_toggle_leaveActive : "",
          enterFromClass: _unref(prefer).s.animation ? _ctx.$style.transition_toggle_enterFrom : "",
          leaveToClass: _unref(prefer).s.animation ? _ctx.$style.transition_toggle_leaveTo : "",
          onEnter: enter,
          onAfterEnter: afterEnter,
          onLeave: leave,
          onAfterLeave: afterLeave
        }, {
          default: _withCtx(() => [_withDirectives(_createElementVNode(
            "div",
            {
              ref_key: "contentEl",
              ref: contentEl,
              class: _normalizeClass([_ctx.$style.content, { [_ctx.$style.omitted]: omitted.value }])
            },
            [_renderSlot(_ctx.$slots, "default"), omitted.value ? (_openBlock(), _createElementBlock(
              "button",
              {
                key: 0,
                class: _normalizeClass(["_button", _ctx.$style.fade]),
                onClick: showMore
              },
              [_createElementVNode(
                "span",
                { class: _normalizeClass(_ctx.$style.fadeLabel) },
                _toDisplayString(_unref(i18n).ts.showMore),
                3
                /* TEXT, CLASS */
              )],
              2
              /* CLASS */
            )) : _createCommentVNode("v-if", true)],
            2
            /* CLASS */
          ), [[_vShow, showBody.value]])]),
          _: 3
        }, 1032, [
          "enterActiveClass",
          "leaveActiveClass",
          "enterFromClass",
          "leaveToClass"
        ])],
        2
        /* CLASS */
      );
    };
  }
};
