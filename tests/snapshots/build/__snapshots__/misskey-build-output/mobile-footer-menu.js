import { useModel as _useModel } from "vue";
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, unref as _unref } from "vue";
const _hoisted_1 = /* @__PURE__ */ _createElementVNode("i", { class: "_indicatorCircle" });
import { computed, ref, useTemplateRef, watch } from "vue";
import { $i } from "@/i.js";
import * as os from "@/os.js";
import { mainRouter } from "@/router.js";
import { navbarItemDef } from "@/navbar.js";
export default {
  __name: "mobile-footer-menu",
  props: {
    "drawerMenuShowing": {},
    "drawerMenuShowingModifiers": {},
    "widgetsShowing": {},
    "widgetsShowingModifiers": {}
  },
  emits: ["update:drawerMenuShowing", "update:widgetsShowing"],
  setup(__props) {
    const drawerMenuShowing = _useModel(__props, "drawerMenuShowing");
    const widgetsShowing = _useModel(__props, "widgetsShowing");
    const rootEl = useTemplateRef("rootEl");
    const menuIndicated = computed(() => {
      for (const def in navbarItemDef) {
        if (def === "notifications") continue;
        if (navbarItemDef[def].indicated) return true;
      }
      return false;
    });
    const rootElHeight = ref(0);
    watch(rootEl, () => {
      if (rootEl.value) {
        rootElHeight.value = rootEl.value.offsetHeight;
        window.document.body.style.setProperty("--MI-minBottomSpacing", "var(--MI-minBottomSpacingMobile)");
      } else {
        rootElHeight.value = 0;
        window.document.body.style.setProperty("--MI-minBottomSpacing", "0px");
      }
    }, { immediate: true });
    return (_ctx, _cache) => {
      return _openBlock(), _createElementBlock(
        "div",
        {
          ref_key: "rootEl",
          ref: rootEl,
          class: _normalizeClass(_ctx.$style.root)
        },
        [
          _createElementVNode(
            "button",
            {
              class: _normalizeClass(["_button", _ctx.$style.item]),
              onClick: _cache[0] || (_cache[0] = ($event) => drawerMenuShowing.value = true)
            },
            [_createElementVNode(
              "div",
              { class: _normalizeClass(_ctx.$style.itemInner) },
              [_createElementVNode(
                "i",
                { class: _normalizeClass(["ti ti-menu-2", _ctx.$style.itemIcon]) },
                null,
                2
                /* CLASS */
              ), menuIndicated.value ? (_openBlock(), _createElementBlock(
                "span",
                {
                  key: 0,
                  class: _normalizeClass(["_blink", _ctx.$style.itemIndicator])
                },
                [_hoisted_1],
                2
                /* CLASS */
              )) : _createCommentVNode("v-if", true)],
              2
              /* CLASS */
            )],
            2
            /* CLASS */
          ),
          _createElementVNode(
            "button",
            {
              class: _normalizeClass(["_button", _ctx.$style.item]),
              onClick: _cache[1] || (_cache[1] = ($event) => _unref(mainRouter).push("/"))
            },
            [_createElementVNode(
              "div",
              { class: _normalizeClass(_ctx.$style.itemInner) },
              [_createElementVNode(
                "i",
                { class: _normalizeClass(["ti ti-home", _ctx.$style.itemIcon]) },
                null,
                2
                /* CLASS */
              )],
              2
              /* CLASS */
            )],
            2
            /* CLASS */
          ),
          _createElementVNode(
            "button",
            {
              class: _normalizeClass(["_button", _ctx.$style.item]),
              onClick: _cache[2] || (_cache[2] = ($event) => _unref(mainRouter).push("/my/notifications"))
            },
            [_createElementVNode(
              "div",
              { class: _normalizeClass(_ctx.$style.itemInner) },
              [_createElementVNode(
                "i",
                { class: _normalizeClass(["ti ti-bell", _ctx.$style.itemIcon]) },
                null,
                2
                /* CLASS */
              ), _unref($i)?.hasUnreadNotification ? (_openBlock(), _createElementBlock(
                "span",
                {
                  key: 0,
                  class: _normalizeClass(["_blink", _ctx.$style.itemIndicator])
                },
                [_createElementVNode(
                  "span",
                  { class: _normalizeClass(["_indicateCounter", _ctx.$style.itemIndicateValueIcon]) },
                  _toDisplayString(_unref($i).unreadNotificationsCount > 99 ? "99+" : _unref($i).unreadNotificationsCount),
                  3
                  /* TEXT, CLASS */
                )],
                2
                /* CLASS */
              )) : _createCommentVNode("v-if", true)],
              2
              /* CLASS */
            )],
            2
            /* CLASS */
          ),
          _createElementVNode(
            "button",
            {
              class: _normalizeClass(["_button", _ctx.$style.item]),
              onClick: _cache[3] || (_cache[3] = ($event) => widgetsShowing.value = true)
            },
            [_createElementVNode(
              "div",
              { class: _normalizeClass(_ctx.$style.itemInner) },
              [_createElementVNode(
                "i",
                { class: _normalizeClass(["ti ti-apps", _ctx.$style.itemIcon]) },
                null,
                2
                /* CLASS */
              )],
              2
              /* CLASS */
            )],
            2
            /* CLASS */
          ),
          _createElementVNode(
            "button",
            {
              class: _normalizeClass(["_button", [_ctx.$style.item, _ctx.$style.post]]),
              onClick: _cache[4] || (_cache[4] = ($event) => os.post())
            },
            [_createElementVNode(
              "div",
              { class: _normalizeClass(_ctx.$style.itemInner) },
              [_createElementVNode(
                "i",
                { class: _normalizeClass(["ti ti-pencil", _ctx.$style.itemIcon]) },
                null,
                2
                /* CLASS */
              )],
              2
              /* CLASS */
            )],
            2
            /* CLASS */
          )
        ],
        2
        /* CLASS */
      );
    };
  }
};
