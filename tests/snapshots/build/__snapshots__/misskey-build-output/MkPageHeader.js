import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, resolveDirective as _resolveDirective, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, unref as _unref, withModifiers as _withModifiers } from "vue";
import { onMounted, onUnmounted, ref, inject, useTemplateRef, computed } from "vue";
import { scrollToTop } from "@@/js/scroll.js";
import XTabs from "./MkPageHeader.tabs.vue";
import { getAccountMenu } from "@/accounts.js";
import { $i } from "@/i.js";
import { DI } from "@/di.js";
import * as os from "@/os.js";
export {};
export default {
  __name: "MkPageHeader",
  props: {
    overridePageMetadata: {
      type: null,
      required: false
    },
    tabs: {
      type: Array,
      required: false,
      default: () => []
    },
    tab: {
      type: String,
      required: false
    },
    actions: {
      type: [Array, null],
      required: false
    },
    thin: {
      type: Boolean,
      required: false
    },
    hideTitle: {
      type: Boolean,
      required: false
    },
    canOmitTitle: {
      type: Boolean,
      required: false
    },
    displayMyAvatar: {
      type: Boolean,
      required: false
    }
  },
  emits: ["update:tab"],
  setup(__props, { emit: __emit }) {
    const emit = __emit;
    const props = __props;
    //const viewId = inject(DI.viewId);
    const injectedPageMetadata = inject(DI.pageMetadata, ref(null));
    const pageMetadata = computed(() => props.overridePageMetadata ?? injectedPageMetadata.value);
    const hideTitle = computed(() => inject("shouldOmitHeaderTitle", false) || props.hideTitle || props.canOmitTitle && props.tabs.length > 0);
    const thin_ = props.thin || inject("shouldHeaderThin", false);
    const el = useTemplateRef("el");
    const narrow = ref(false);
    const hasTabs = computed(() => props.tabs.length > 0);
    const hasActions = computed(() => props.actions && props.actions.length > 0);
    const show = computed(() => {
      return !hideTitle.value || hasTabs.value || hasActions.value;
    });
    const preventDrag = (ev) => {
      ev.stopPropagation();
    };
    const top = () => {
      if (el.value) {
        scrollToTop(el.value, { behavior: "smooth" });
      }
    };
    async function openAccountMenu(ev) {
      const menuItems = await getAccountMenu({ withExtraOperation: true });
      os.popupMenu(menuItems, ev.currentTarget ?? ev.target);
    }
    function onTabClick() {
      top();
    }
    let ro;
    onMounted(() => {
      if (el.value && el.value.parentElement) {
        narrow.value = el.value.parentElement.offsetWidth < 500;
        ro = new ResizeObserver((entries, observer) => {
          if (el.value && el.value.parentElement && window.document.body.contains(el.value)) {
            narrow.value = el.value.parentElement.offsetWidth < 500;
          }
        });
        ro.observe(el.value.parentElement);
      }
    });
    onUnmounted(() => {
      if (ro) ro.disconnect();
    });
    return (_ctx, _cache) => {
      const _component_MkAvatar = _resolveComponent("MkAvatar");
      const _component_MkUserName = _resolveComponent("MkUserName");
      const _directive_tooltip = _resolveDirective("tooltip");
      return show.value ? (_openBlock(), _createElementBlock(
        "div",
        {
          key: 0,
          ref_key: "el",
          ref: el,
          class: _normalizeClass([_ctx.$style.root])
        },
        [_createElementVNode(
          "div",
          { class: _normalizeClass([_ctx.$style.upper, {
            [_ctx.$style.slim]: narrow.value,
            [_ctx.$style.thin]: _unref(thin_)
          }]) },
          [
            !_unref(thin_) && narrow.value && props.displayMyAvatar && _unref($i) ? (_openBlock(), _createElementBlock("div", {
              key: 0,
              class: "_button",
              onClick: openAccountMenu
            }, [_createVNode(_component_MkAvatar, {
              class: _normalizeClass(_ctx.$style.avatar),
              user: _unref($i)
            }, null, 10, ["user"])])) : !_unref(thin_) && narrow.value && !hideTitle.value ? (_openBlock(), _createElementBlock(
              "div",
              {
                key: 1,
                class: _normalizeClass(_ctx.$style.buttons)
              },
              null,
              2
              /* CLASS */
            )) : _createCommentVNode("v-if", true),
            pageMetadata.value ? (_openBlock(), _createElementBlock(
              _Fragment,
              { key: 0 },
              [!hideTitle.value ? (_openBlock(), _createElementBlock(
                "div",
                {
                  key: 0,
                  class: _normalizeClass(_ctx.$style.titleContainer),
                  onClick: top
                },
                [pageMetadata.value.avatar ? (_openBlock(), _createElementBlock(
                  "div",
                  {
                    key: 0,
                    class: _normalizeClass(_ctx.$style.titleAvatarContainer)
                  },
                  [_createVNode(_component_MkAvatar, {
                    class: _normalizeClass(_ctx.$style.titleAvatar),
                    user: pageMetadata.value.avatar,
                    indicator: ""
                  }, null, 10, ["user"])],
                  2
                  /* CLASS */
                )) : pageMetadata.value.icon ? (_openBlock(), _createElementBlock(
                  "i",
                  {
                    key: 1,
                    class: _normalizeClass([_ctx.$style.titleIcon, pageMetadata.value.icon])
                  },
                  null,
                  2
                  /* CLASS */
                )) : _createCommentVNode("v-if", true), _createElementVNode(
                  "div",
                  { class: _normalizeClass(["_nowrap", _ctx.$style.title]) },
                  [pageMetadata.value.userName ? (_openBlock(), _createBlock(_component_MkUserName, {
                    key: 0,
                    user: pageMetadata.value.userName,
                    nowrap: true
                  }, null, 8, ["user", "nowrap"])) : pageMetadata.value.title ? (_openBlock(), _createElementBlock(
                    "div",
                    {
                      key: 1,
                      class: "_nowrap"
                    },
                    _toDisplayString(pageMetadata.value.title),
                    1
                    /* TEXT */
                  )) : _createCommentVNode("v-if", true), pageMetadata.value.subtitle ? (_openBlock(), _createElementBlock(
                    "div",
                    {
                      key: 0,
                      class: _normalizeClass(_ctx.$style.subtitle)
                    },
                    _toDisplayString(pageMetadata.value.subtitle),
                    3
                    /* TEXT, CLASS */
                  )) : _createCommentVNode("v-if", true)],
                  2
                  /* CLASS */
                )],
                2
                /* CLASS */
              )) : _createCommentVNode("v-if", true), !narrow.value || hideTitle.value ? (_openBlock(), _createBlock(XTabs, {
                key: 0,
                class: _normalizeClass(_ctx.$style.tabs),
                tab: __props.tab,
                tabs: __props.tabs,
                rootEl: _unref(el),
                "onUpdate:tab": _cache[0] || (_cache[0] = (key) => emit("update:tab", key)),
                onTabClick
              }, null, 10, [
                "tab",
                "tabs",
                "rootEl"
              ])) : _createCommentVNode("v-if", true)],
              64
              /* STABLE_FRAGMENT */
            )) : _createCommentVNode("v-if", true),
            !_unref(thin_) && narrow.value && !hideTitle.value || __props.actions && __props.actions.length > 0 ? (_openBlock(), _createElementBlock(
              "div",
              {
                key: 0,
                class: _normalizeClass(_ctx.$style.buttons)
              },
              [(_openBlock(true), _createElementBlock(
                _Fragment,
                null,
                _renderList(__props.actions, (action) => {
                  return _openBlock(), _createElementBlock("button", {
                    class: _normalizeClass(["_button", [_ctx.$style.button, { [_ctx.$style.highlighted]: action.highlighted }]]),
                    onClick: _withModifiers(action.handler, ["stop"]),
                    onTouchstart: preventDrag
                  }, [_createElementVNode(
                    "i",
                    { class: _normalizeClass(action.icon) },
                    null,
                    2
                    /* CLASS */
                  )], 42, ["onClick"]);
                }),
                256
                /* UNKEYED_FRAGMENT */
              ))],
              2
              /* CLASS */
            )) : _createCommentVNode("v-if", true)
          ],
          2
          /* CLASS */
        ), narrow.value && !hideTitle.value && hasTabs.value ? (_openBlock(), _createElementBlock(
          "div",
          {
            key: 0,
            class: _normalizeClass([_ctx.$style.lower, {
              [_ctx.$style.slim]: narrow.value,
              [_ctx.$style.thin]: _unref(thin_)
            }])
          },
          [_createVNode(XTabs, {
            class: _normalizeClass(_ctx.$style.tabs),
            tab: __props.tab,
            tabs: __props.tabs,
            rootEl: _unref(el),
            "onUpdate:tab": _cache[1] || (_cache[1] = (key) => emit("update:tab", key)),
            onTabClick
          }, null, 10, [
            "tab",
            "tabs",
            "rootEl"
          ])],
          2
          /* CLASS */
        )) : _createCommentVNode("v-if", true)],
        2
        /* CLASS */
      )) : _createCommentVNode("v-if", true);
    };
  }
};
