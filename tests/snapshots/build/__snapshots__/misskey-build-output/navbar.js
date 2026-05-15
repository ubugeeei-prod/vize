import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, resolveDynamicComponent as _resolveDynamicComponent, resolveDirective as _resolveDirective, withDirectives as _withDirectives, renderList as _renderList, toDisplayString as _toDisplayString, mergeProps as _mergeProps, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, toHandlers as _toHandlers, withCtx as _withCtx, unref as _unref } from "vue";
const _hoisted_1 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-apps ti-fw" });
const _hoisted_2 = /* @__PURE__ */ _createElementVNode("i", { class: "_indicatorCircle" });
const _hoisted_3 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-apps ti-fw" });
const _hoisted_4 = /* @__PURE__ */ _createElementVNode("path", {
  d: "M47.488,7.995C47.79,10.11 47.943,12.266 47.943,14.429C47.997,26.989 47.997,84 47.997,84C47.997,84 44.018,118.246 23.997,133.5C-0.374,152.07 -0.003,192 -0.003,192L-0.003,-96C-0.003,-96 0.151,-56.216 23.997,-37.5C40.861,-24.265 46.043,-1.243 47.488,7.995Z",
  style: "fill:var(--MI_THEME-navBg);"
});
const _hoisted_5 = /* @__PURE__ */ _createElementVNode("path", {
  d: "M47.488,7.995C47.79,10.11 47.943,12.266 47.943,14.429C47.997,26.989 47.997,84 47.997,84C47.997,84 44.018,118.246 23.997,133.5C-0.374,152.07 -0.003,192 -0.003,192L-0.003,-96C-0.003,-96 0.151,-56.216 23.997,-37.5C40.861,-24.265 46.043,-1.243 47.488,7.995Z",
  style: "fill:var(--MI_THEME-navBg);"
});
import { computed, ref, watch } from "vue";
import { openInstanceMenu } from "./common.js";
import * as os from "@/os.js";
import { navbarItemDef } from "@/navbar.js";
import { store } from "@/store.js";
import { i18n } from "@/i18n.js";
import { instance } from "@/instance.js";
import { getHTMLElementOrNull } from "@/utility/get-dom-node-or-null.js";
import { useRouter } from "@/router.js";
import { prefer } from "@/preferences.js";
import { getAccountMenu } from "@/accounts.js";
import { $i } from "@/i.js";
export default {
  __name: "navbar",
  props: {
    showWidgetButton: {
      type: Boolean,
      required: false
    },
    asDrawer: {
      type: Boolean,
      required: false
    }
  },
  emits: ["widgetButtonClick"],
  setup(__props, { emit: __emit }) {
    const emit = __emit;
    const props = __props;
    const router = useRouter();
    const forceIconOnly = ref(!props.asDrawer && window.innerWidth <= 1279);
    const iconOnly = computed(() => {
      return !props.asDrawer && (forceIconOnly.value || store.r.menuDisplay.value === "sideIcon");
    });
    const otherMenuItemIndicated = computed(() => {
      for (const def in navbarItemDef) {
        if (prefer.r.menu.value.includes(def)) continue;
        if (navbarItemDef[def].indicated) return true;
      }
      return false;
    });
    function calcViewState() {
      forceIconOnly.value = window.innerWidth <= 1279;
    }
    window.addEventListener("resize", calcViewState);
    watch(store.r.menuDisplay, () => {
      calcViewState();
    });
    function toggleIconOnly() {
      if (window.document.startViewTransition && prefer.s.animation) {
        window.document.startViewTransition(() => {
          store.set("menuDisplay", iconOnly.value ? "sideFull" : "sideIcon");
        });
      } else {
        store.set("menuDisplay", iconOnly.value ? "sideFull" : "sideIcon");
      }
    }
    function toggleRealtimeMode(ev) {
      os.popupMenu([{
        type: "label",
        text: i18n.ts.realtimeMode
      }, {
        text: store.s.realtimeMode ? i18n.ts.turnItOff : i18n.ts.turnItOn,
        icon: store.s.realtimeMode ? "ti ti-bolt-off" : "ti ti-bolt",
        action: () => {
          store.set("realtimeMode", !store.s.realtimeMode);
          window.location.reload();
        }
      }], ev.currentTarget ?? ev.target);
    }
    async function openAccountMenu(ev) {
      const menuItems = await getAccountMenu({ withExtraOperation: true });
      os.popupMenu(menuItems, ev.currentTarget ?? ev.target);
    }
    async function more(ev) {
      const target = getHTMLElementOrNull(ev.currentTarget ?? ev.target);
      if (!target) return;
      const { dispose } = await os.popupAsyncWithDialog(import("@/components/MkLaunchPad.vue").then((x) => x.default), { anchorElement: target }, { closed: () => dispose() });
    }
    function menuEdit() {
      router.push("/settings/navbar");
    }
    return (_ctx, _cache) => {
      const _component_MkA = _resolveComponent("MkA");
      const _component_MkAvatar = _resolveComponent("MkAvatar");
      const _component_MkAcct = _resolveComponent("MkAcct");
      const _directive_tooltip = _resolveDirective("tooltip");
      return _openBlock(), _createElementBlock(
        "div",
        { class: _normalizeClass([_ctx.$style.root, { [_ctx.$style.iconOnly]: iconOnly.value }]) },
        [_createElementVNode(
          "div",
          { class: _normalizeClass(_ctx.$style.body) },
          [
            _createElementVNode(
              "div",
              { class: _normalizeClass(_ctx.$style.top) },
              [
                _withDirectives(_createElementVNode(
                  "button",
                  {
                    class: _normalizeClass(["_button", _ctx.$style.instance]),
                    onClick: _cache[0] || (_cache[0] = (...args) => openInstanceMenu && openInstanceMenu(...args))
                  },
                  [_createElementVNode("img", {
                    src: _unref(instance).iconUrl || "/favicon.ico",
                    alt: "",
                    class: _normalizeClass(_ctx.$style.instanceIcon),
                    style: "view-transition-name: navbar-serverIcon;"
                  }, null, 10, ["src"])],
                  2
                  /* CLASS */
                ), [[
                  _directive_tooltip,
                  _unref(instance).name ?? _unref(i18n).ts.instance,
                  void 0,
                  {
                    noDelay: true,
                    right: true
                  }
                ]]),
                !iconOnly.value ? _withDirectives((_openBlock(), _createElementBlock(
                  "button",
                  {
                    key: 0,
                    class: _normalizeClass(["_button", [_ctx.$style.realtimeMode, _unref(store).r.realtimeMode.value ? _ctx.$style.on : null]]),
                    onClick: toggleRealtimeMode
                  },
                  [_unref(store).r.realtimeMode.value ? (_openBlock(), _createElementBlock("i", {
                    key: 0,
                    class: "ti ti-bolt ti-fw"
                  })) : (_openBlock(), _createElementBlock("i", {
                    key: 1,
                    class: "ti ti-bolt-off ti-fw"
                  }))],
                  2
                  /* CLASS */
                )), [[
                  _directive_tooltip,
                  _unref(i18n).ts.realtimeMode,
                  void 0,
                  {
                    noDelay: true,
                    right: true
                  }
                ]]) : _createCommentVNode("v-if", true),
                !iconOnly.value && __props.showWidgetButton ? _withDirectives((_openBlock(), _createElementBlock(
                  "button",
                  {
                    key: 0,
                    class: _normalizeClass(["_button", [_ctx.$style.widget]]),
                    onClick: _cache[1] || (_cache[1] = () => emit("widgetButtonClick"))
                  },
                  [_hoisted_1],
                  2
                  /* CLASS */
                )), [[
                  _directive_tooltip,
                  _unref(i18n).ts.widgets,
                  void 0,
                  {
                    noDelay: true,
                    right: true
                  }
                ]]) : _createCommentVNode("v-if", true)
              ],
              2
              /* CLASS */
            ),
            _createElementVNode(
              "div",
              { class: _normalizeClass(_ctx.$style.middle) },
              [
                _withDirectives(_createVNode(_component_MkA, {
                  class: _normalizeClass(_ctx.$style.item),
                  activeClass: _ctx.$style.active,
                  to: "/",
                  exact: ""
                }, {
                  default: _withCtx(() => [_createElementVNode(
                    "i",
                    {
                      class: _normalizeClass(["ti ti-home ti-fw", _ctx.$style.itemIcon]),
                      style: "view-transition-name: navbar-homeIcon;"
                    },
                    null,
                    2
                    /* CLASS */
                  ), _createElementVNode(
                    "span",
                    { class: _normalizeClass(_ctx.$style.itemText) },
                    _toDisplayString(_unref(i18n).ts.timeline),
                    3
                    /* TEXT, CLASS */
                  )]),
                  _: 1
                }, 10, ["activeClass"]), [[
                  _directive_tooltip,
                  _unref(i18n).ts.timeline,
                  void 0,
                  {
                    noDelay: true,
                    right: true
                  }
                ]]),
                (_openBlock(true), _createElementBlock(
                  _Fragment,
                  null,
                  _renderList(_unref(prefer).r.menu.value, (item) => {
                    return _openBlock(), _createElementBlock(
                      _Fragment,
                      null,
                      [item === "-" ? (_openBlock(), _createElementBlock(
                        "div",
                        {
                          key: 0,
                          class: _normalizeClass(_ctx.$style.divider)
                        },
                        null,
                        2
                        /* CLASS */
                      )) : _unref(navbarItemDef)[item] && (_unref(navbarItemDef)[item].show == null || _unref(navbarItemDef)[item].show.value !== false) ? _withDirectives((_openBlock(), _createBlock(_resolveDynamicComponent(_unref(navbarItemDef)[item].to ? "MkA" : "button"), _mergeProps(_toHandlers(_unref(navbarItemDef)[item].action ? { click: _unref(navbarItemDef)[item].action } : {}, true), {
                        key: 1,
                        is: _unref(navbarItemDef)[item].to ? "MkA" : "button",
                        class: _normalizeClass(["_button", [_ctx.$style.item]]),
                        activeClass: _ctx.$style.active,
                        to: _unref(navbarItemDef)[item].to
                      }), {
                        default: _withCtx(() => [
                          _createElementVNode(
                            "i",
                            {
                              class: _normalizeClass(["ti-fw", [_ctx.$style.itemIcon, _unref(navbarItemDef)[item].icon]]),
                              style: _normalizeStyle({ viewTransitionName: "navbar-item-" + item })
                            },
                            null,
                            6
                            /* CLASS, STYLE */
                          ),
                          _createElementVNode(
                            "span",
                            { class: _normalizeClass(_ctx.$style.itemText) },
                            _toDisplayString(_unref(navbarItemDef)[item].title),
                            3
                            /* TEXT, CLASS */
                          ),
                          _unref(navbarItemDef)[item].indicated ? (_openBlock(), _createElementBlock(
                            "span",
                            {
                              key: 0,
                              class: _normalizeClass(["_blink", _ctx.$style.itemIndicator])
                            },
                            [_unref(navbarItemDef)[item].indicateValue ? (_openBlock(), _createElementBlock(
                              "span",
                              {
                                key: 0,
                                class: _normalizeClass(["_indicateCounter", _ctx.$style.itemIndicateValueIcon])
                              },
                              _toDisplayString(_unref(navbarItemDef)[item].indicateValue),
                              3
                              /* TEXT, CLASS */
                            )) : (_openBlock(), _createElementBlock("i", {
                              key: 1,
                              class: "_indicatorCircle"
                            }))],
                            2
                            /* CLASS */
                          )) : _createCommentVNode("v-if", true)
                        ]),
                        _: 2
                      }, 1040, ["activeClass", "to"])), [[
                        _directive_tooltip,
                        _unref(navbarItemDef)[item].title,
                        void 0,
                        {
                          noDelay: true,
                          right: true
                        }
                      ]]) : _createCommentVNode("v-if", true)],
                      64
                      /* STABLE_FRAGMENT */
                    );
                  }),
                  256
                  /* UNKEYED_FRAGMENT */
                )),
                _createElementVNode(
                  "div",
                  { class: _normalizeClass(_ctx.$style.divider) },
                  null,
                  2
                  /* CLASS */
                ),
                _unref($i) != null && (_unref($i).isAdmin || _unref($i).isModerator) ? _withDirectives((_openBlock(), _createBlock(_component_MkA, {
                  key: 0,
                  class: _normalizeClass(_ctx.$style.item),
                  activeClass: _ctx.$style.active,
                  to: "/admin"
                }, {
                  default: _withCtx(() => [_createElementVNode(
                    "i",
                    {
                      class: _normalizeClass(["ti ti-dashboard ti-fw", _ctx.$style.itemIcon]),
                      style: "view-transition-name: navbar-controlPanel;"
                    },
                    null,
                    2
                    /* CLASS */
                  ), _createElementVNode(
                    "span",
                    { class: _normalizeClass(_ctx.$style.itemText) },
                    _toDisplayString(_unref(i18n).ts.controlPanel),
                    3
                    /* TEXT, CLASS */
                  )]),
                  _: 1
                }, 10, ["activeClass"])), [[
                  _directive_tooltip,
                  _unref(i18n).ts.controlPanel,
                  void 0,
                  {
                    noDelay: true,
                    right: true
                  }
                ]]) : _createCommentVNode("v-if", true),
                _createElementVNode(
                  "button",
                  {
                    class: _normalizeClass(["_button", _ctx.$style.item]),
                    onClick: more
                  },
                  [
                    _createElementVNode(
                      "i",
                      {
                        class: _normalizeClass(["ti ti-grid-dots ti-fw", _ctx.$style.itemIcon]),
                        style: "view-transition-name: navbar-more;"
                      },
                      null,
                      2
                      /* CLASS */
                    ),
                    _createElementVNode(
                      "span",
                      { class: _normalizeClass(_ctx.$style.itemText) },
                      _toDisplayString(_unref(i18n).ts.more),
                      3
                      /* TEXT, CLASS */
                    ),
                    otherMenuItemIndicated.value ? (_openBlock(), _createElementBlock(
                      "span",
                      {
                        key: 0,
                        class: _normalizeClass(["_blink", _ctx.$style.itemIndicator])
                      },
                      [_hoisted_2],
                      2
                      /* CLASS */
                    )) : _createCommentVNode("v-if", true)
                  ],
                  2
                  /* CLASS */
                ),
                _withDirectives(_createVNode(_component_MkA, {
                  class: _normalizeClass(_ctx.$style.item),
                  activeClass: _ctx.$style.active,
                  to: "/settings"
                }, {
                  default: _withCtx(() => [_createElementVNode(
                    "i",
                    {
                      class: _normalizeClass(["ti ti-settings ti-fw", _ctx.$style.itemIcon]),
                      style: "view-transition-name: navbar-settings;"
                    },
                    null,
                    2
                    /* CLASS */
                  ), _createElementVNode(
                    "span",
                    { class: _normalizeClass(_ctx.$style.itemText) },
                    _toDisplayString(_unref(i18n).ts.settings),
                    3
                    /* TEXT, CLASS */
                  )]),
                  _: 1
                }, 10, ["activeClass"]), [[
                  _directive_tooltip,
                  _unref(i18n).ts.settings,
                  void 0,
                  {
                    noDelay: true,
                    right: true
                  }
                ]])
              ],
              2
              /* CLASS */
            ),
            _createElementVNode(
              "div",
              { class: _normalizeClass(_ctx.$style.bottom) },
              [
                iconOnly.value && __props.showWidgetButton ? _withDirectives((_openBlock(), _createElementBlock(
                  "button",
                  {
                    key: 0,
                    class: _normalizeClass(["_button", [_ctx.$style.widget]]),
                    onClick: _cache[2] || (_cache[2] = () => emit("widgetButtonClick"))
                  },
                  [_hoisted_3],
                  2
                  /* CLASS */
                )), [[
                  _directive_tooltip,
                  _unref(i18n).ts.widgets,
                  void 0,
                  {
                    noDelay: true,
                    right: true
                  }
                ]]) : _createCommentVNode("v-if", true),
                iconOnly.value ? _withDirectives((_openBlock(), _createElementBlock(
                  "button",
                  {
                    key: 0,
                    class: _normalizeClass(["_button", [_ctx.$style.realtimeMode, _unref(store).r.realtimeMode.value ? _ctx.$style.on : null]]),
                    onClick: toggleRealtimeMode
                  },
                  [_unref(store).r.realtimeMode.value ? (_openBlock(), _createElementBlock("i", {
                    key: 0,
                    class: "ti ti-bolt ti-fw"
                  })) : (_openBlock(), _createElementBlock("i", {
                    key: 1,
                    class: "ti ti-bolt-off ti-fw"
                  }))],
                  2
                  /* CLASS */
                )), [[
                  _directive_tooltip,
                  _unref(i18n).ts.realtimeMode,
                  void 0,
                  {
                    noDelay: true,
                    right: true
                  }
                ]]) : _createCommentVNode("v-if", true),
                _withDirectives(_createElementVNode(
                  "button",
                  {
                    class: _normalizeClass(["_button", [_ctx.$style.post]]),
                    "data-cy-open-post-form": "",
                    onClick: _cache[3] || (_cache[3] = () => {
                      os.post();
                    })
                  },
                  [_createElementVNode(
                    "i",
                    { class: _normalizeClass(["ti ti-pencil ti-fw", _ctx.$style.postIcon]) },
                    null,
                    2
                    /* CLASS */
                  ), _createElementVNode(
                    "span",
                    { class: _normalizeClass(_ctx.$style.postText) },
                    _toDisplayString(_unref(i18n).ts.note),
                    3
                    /* TEXT, CLASS */
                  )],
                  2
                  /* CLASS */
                ), [[
                  _directive_tooltip,
                  _unref(i18n).ts.note,
                  void 0,
                  {
                    noDelay: true,
                    right: true
                  }
                ]]),
                _unref($i) != null ? _withDirectives((_openBlock(), _createElementBlock(
                  "button",
                  {
                    key: 0,
                    class: _normalizeClass(["_button", [_ctx.$style.account]]),
                    onClick: openAccountMenu
                  },
                  [_createVNode(_component_MkAvatar, {
                    user: _unref($i),
                    class: _normalizeClass(_ctx.$style.avatar),
                    style: "view-transition-name: navbar-avatar;"
                  }, null, 10, ["user"]), _createVNode(_component_MkAcct, {
                    class: _normalizeClass(["_nowrap", _ctx.$style.acct]),
                    user: _unref($i)
                  }, null, 10, ["user"])],
                  2
                  /* CLASS */
                )), [[
                  _directive_tooltip,
                  `${_unref(i18n).ts.account}: @${_unref($i).username}`,
                  void 0,
                  {
                    noDelay: true,
                    right: true
                  }
                ]]) : _createCommentVNode("v-if", true)
              ],
              2
              /* CLASS */
            )
          ],
          2
          /* CLASS */
        ), !forceIconOnly.value && _unref(prefer).r.showNavbarSubButtons.value ? (_openBlock(), _createElementBlock(
          "div",
          {
            key: 0,
            class: _normalizeClass(_ctx.$style.subButtons)
          },
          [_createElementVNode(
            "div",
            { class: _normalizeClass(_ctx.$style.subButton) },
            [_createElementVNode(
              "svg",
              {
                viewBox: "0 0 16 64",
                class: _normalizeClass(_ctx.$style.subButtonShape)
              },
              [_createElementVNode("g", { transform: "matrix(0.333333,0,0,0.222222,0.000895785,21.3333)" }, [_hoisted_4])],
              2
              /* CLASS */
            ), _createElementVNode(
              "button",
              {
                class: _normalizeClass(["_button", _ctx.$style.subButtonClickable]),
                onClick: menuEdit
              },
              [_createElementVNode(
                "i",
                { class: _normalizeClass(["ti ti-settings-2", _ctx.$style.subButtonIcon]) },
                null,
                2
                /* CLASS */
              )],
              2
              /* CLASS */
            )],
            2
            /* CLASS */
          ), !props.asDrawer ? (_openBlock(), _createElementBlock(
            _Fragment,
            { key: 0 },
            [
              _createElementVNode(
                "div",
                { class: _normalizeClass(_ctx.$style.subButtonGapFill) },
                null,
                2
                /* CLASS */
              ),
              _createElementVNode(
                "div",
                { class: _normalizeClass(_ctx.$style.subButtonGapFillDivider) },
                null,
                2
                /* CLASS */
              ),
              _createElementVNode(
                "div",
                { class: _normalizeClass(_ctx.$style.subButton) },
                [_createElementVNode(
                  "svg",
                  {
                    viewBox: "0 0 16 64",
                    class: _normalizeClass(_ctx.$style.subButtonShape)
                  },
                  [_createElementVNode("g", { transform: "matrix(0.333333,0,0,0.222222,0.000895785,21.3333)" }, [_hoisted_5])],
                  2
                  /* CLASS */
                ), _createElementVNode(
                  "button",
                  {
                    class: _normalizeClass(["_button", _ctx.$style.subButtonClickable]),
                    onClick: toggleIconOnly
                  },
                  [iconOnly.value ? (_openBlock(), _createElementBlock(
                    "i",
                    {
                      key: 0,
                      class: _normalizeClass(["ti ti-chevron-right", _ctx.$style.subButtonIcon])
                    },
                    null,
                    2
                    /* CLASS */
                  )) : (_openBlock(), _createElementBlock(
                    "i",
                    {
                      key: 1,
                      class: _normalizeClass(["ti ti-chevron-left", _ctx.$style.subButtonIcon])
                    },
                    null,
                    2
                    /* CLASS */
                  ))],
                  2
                  /* CLASS */
                )],
                2
                /* CLASS */
              )
            ],
            64
            /* STABLE_FRAGMENT */
          )) : _createCommentVNode("v-if", true)],
          2
          /* CLASS */
        )) : _createCommentVNode("v-if", true)],
        2
        /* CLASS */
      );
    };
  }
};
