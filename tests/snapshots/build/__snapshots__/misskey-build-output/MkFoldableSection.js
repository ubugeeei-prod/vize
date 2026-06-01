import { Fragment as _Fragment, Transition as _Transition, openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, withDirectives as _withDirectives, renderSlot as _renderSlot, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref, vShow as _vShow } from "vue";
const _hoisted_1 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-chevron-up" });
const _hoisted_2 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-chevron-down" });
import { onBeforeUnmount, onMounted, ref, useTemplateRef, watch } from "vue";
import { miLocalStorage } from "@/local-storage.js";
import { prefer } from "@/preferences.js";
import { globalEvents } from "@/events.js";
import { getBgColor } from "@/utility/get-bg-color.js";
export default {
  __name: "MkFoldableSection",
  props: {
    expanded: {
      type: Boolean,
      required: false,
      default: true
    },
    persistKey: {
      type: [String, null],
      required: false,
      default: null
    }
  },
  setup(__props) {
    const props = __props;
    const miLocalStoragePrefix = "ui:folder:";
    const rootEl = useTemplateRef("rootEl");
    const parentBg = ref(null);
    // eslint-disable-next-line vue/no-setup-props-reactivity-loss
    const showBody = ref(props.persistKey && miLocalStorage.getItem(`${miLocalStoragePrefix}${props.persistKey}`) ? miLocalStorage.getItem(`${miLocalStoragePrefix}${props.persistKey}`) === "t" : props.expanded);
    watch(showBody, () => {
      if (props.persistKey) {
        miLocalStorage.setItem(`${miLocalStoragePrefix}${props.persistKey}`, showBody.value ? "t" : "f");
      }
    });
    function enter(el) {
      if (!(el instanceof HTMLElement)) return;
      const elementHeight = el.getBoundingClientRect().height;
      el.style.height = "0";
      el.offsetHeight;
      el.style.height = `${elementHeight}px`;
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
    function updateBgColor() {
      if (rootEl.value) {
        parentBg.value = getBgColor(rootEl.value.parentElement);
      }
    }
    onMounted(() => {
      updateBgColor();
      globalEvents.on("themeChanging", updateBgColor);
    });
    onBeforeUnmount(() => {
      globalEvents.off("themeChanging", updateBgColor);
    });
    return (_ctx, _cache) => {
      return _openBlock(), _createElementBlock(
        "div",
        {
          ref_key: "rootEl",
          ref: rootEl,
          class: _normalizeClass(_ctx.$style.root)
        },
        [_createElementVNode(
          "header",
          {
            class: _normalizeClass(["_button", _ctx.$style.header]),
            onClick: _cache[0] || (_cache[0] = ($event) => showBody.value = !showBody.value)
          },
          [
            _createElementVNode(
              "div",
              { class: _normalizeClass(_ctx.$style.title) },
              [_createElementVNode("div", null, [_renderSlot(_ctx.$slots, "header")])],
              2
              /* CLASS */
            ),
            _createElementVNode(
              "div",
              { class: _normalizeClass(_ctx.$style.divider) },
              null,
              2
              /* CLASS */
            ),
            _createElementVNode(
              "button",
              { class: _normalizeClass(["_button", _ctx.$style.button]) },
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
            )
          ],
          2
          /* CLASS */
        ), _createVNode(_Transition, {
          enterActiveClass: _unref(prefer).s.animation ? _ctx.$style.folderToggleEnterActive : "",
          leaveActiveClass: _unref(prefer).s.animation ? _ctx.$style.folderToggleLeaveActive : "",
          enterFromClass: _unref(prefer).s.animation ? _ctx.$style.folderToggleEnterFrom : "",
          leaveToClass: _unref(prefer).s.animation ? _ctx.$style.folderToggleLeaveTo : "",
          onEnter: enter,
          onAfterEnter: afterEnter,
          onLeave: leave,
          onAfterLeave: afterLeave
        }, {
          default: _withCtx(() => [_withDirectives(_createElementVNode(
            "div",
            null,
            [_renderSlot(_ctx.$slots, "default")],
            512
            /* NEED_PATCH */
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
