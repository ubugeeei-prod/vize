import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, resolveDirective as _resolveDirective, withDirectives as _withDirectives, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, withCtx as _withCtx, unref as _unref, withModifiers as _withModifiers } from "vue";
const _hoisted_1 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-dots" });
const _hoisted_2 = /* @__PURE__ */ _createElementVNode("div");
import { isLink } from "@@/js/is-link.js";
import { i18n } from "@/i18n.js";
import MkButton from "@/components/MkButton.vue";
import bytes from "@/filters/bytes.js";
export default {
  __name: "MkUploaderItems",
  props: { items: {
    type: Array,
    required: true
  } },
  emits: ["showMenu", "showMenuViaContextmenu"],
  setup(__props, { emit: __emit }) {
    const emit = __emit;
    const props = __props;
    function onContextmenu(item, ev) {
      if (ev.target && isLink(ev.target)) return;
      if (window.getSelection()?.toString() !== "") return;
      emit("showMenuViaContextmenu", item, ev);
    }
    function onThumbnailClick(item, ev) {
      // TODO: preview when item is image
    }
    return (_ctx, _cache) => {
      const _component_MkCondensedLine = _resolveComponent("MkCondensedLine");
      const _component_MkLoading = _resolveComponent("MkLoading");
      const _component_MkSystemIcon = _resolveComponent("MkSystemIcon");
      const _directive_panel = _resolveDirective("panel");
      return _openBlock(), _createElementBlock(
        "div",
        { class: _normalizeClass(["_gaps_s", _ctx.$style.root]) },
        [(_openBlock(true), _createElementBlock(
          _Fragment,
          null,
          _renderList(props.items, (item) => {
            return _withDirectives((_openBlock(), _createElementBlock("div", {
              key: item.id,
              class: _normalizeClass([_ctx.$style.item, {
                [_ctx.$style.itemWaiting]: item.preprocessing,
                [_ctx.$style.itemCompleted]: item.uploaded,
                [_ctx.$style.itemFailed]: item.uploadFailed
              }]),
              style: _normalizeStyle({
                "--p": item.progress != null ? `${item.progress.value / item.progress.max * 100}%` : "0%",
                "--pp": item.preprocessProgress != null ? `${item.preprocessProgress * 100}%` : "100%"
              }),
              onContextmenu: _withModifiers(($event) => onContextmenu(item, $event), ["prevent", "stop"])
            }, [_createElementVNode(
              "div",
              { class: _normalizeClass(_ctx.$style.itemInner) },
              [
                _createElementVNode(
                  "div",
                  { class: _normalizeClass(_ctx.$style.itemActionWrapper) },
                  [_createVNode(MkButton, {
                    iconOnly: true,
                    rounded: "",
                    onClick: ($event) => emit("showMenu", item, $event)
                  }, {
                    default: _withCtx(() => [_hoisted_1]),
                    _: 2
                  }, 8, ["iconOnly", "onClick"])],
                  2
                  /* CLASS */
                ),
                _createElementVNode("div", {
                  class: _normalizeClass(_ctx.$style.itemThumbnail),
                  style: _normalizeStyle({ backgroundImage: `url(${item.thumbnail})` }),
                  onClick: ($event) => onThumbnailClick(item, $event)
                }, null, 14, ["onClick"]),
                _createElementVNode(
                  "div",
                  { class: _normalizeClass(_ctx.$style.itemBody) },
                  [
                    _createElementVNode("div", null, [item.isSensitive ? (_openBlock(), _createElementBlock("i", {
                      key: 0,
                      style: "color: var(--MI_THEME-warn); margin-right: 0.5em;",
                      class: "ti ti-eye-exclamation"
                    })) : _createCommentVNode("v-if", true), _createVNode(_component_MkCondensedLine, { minScale: 2 / 3 }, {
                      default: _withCtx(() => [_createTextVNode(
                        _toDisplayString(item.name),
                        1
                        /* TEXT */
                      )]),
                      _: 2
                    }, 8, ["minScale"])]),
                    _createElementVNode(
                      "div",
                      { class: _normalizeClass(_ctx.$style.itemInfo) },
                      [
                        _createElementVNode(
                          "span",
                          null,
                          _toDisplayString(item.file.type),
                          1
                          /* TEXT */
                        ),
                        item.compressedSize ? (_openBlock(), _createElementBlock(
                          "span",
                          { key: 0 },
                          "(" + _toDisplayString(_unref(i18n).tsx._uploader.compressedToX({ x: bytes(item.compressedSize) })) + " = " + _toDisplayString(_unref(i18n).tsx._uploader.savedXPercent({ x: Math.round((1 - item.compressedSize / item.file.size) * 100) })) + ")",
                          1
                          /* TEXT */
                        )) : (_openBlock(), _createElementBlock(
                          "span",
                          { key: 1 },
                          _toDisplayString(bytes(item.file.size)),
                          1
                          /* TEXT */
                        )),
                        item.preprocessing ? (_openBlock(), _createElementBlock("span", { key: 0 }, [_createTextVNode(
                          _toDisplayString(_unref(i18n).ts.preprocessing),
                          1
                          /* TEXT */
                        ), _createVNode(_component_MkLoading, {
                          inline: "",
                          em: "",
                          style: "margin-left: 0.5em;"
                        })])) : _createCommentVNode("v-if", true)
                      ],
                      2
                      /* CLASS */
                    ),
                    _hoisted_2
                  ],
                  2
                  /* CLASS */
                ),
                _createElementVNode(
                  "div",
                  { class: _normalizeClass(_ctx.$style.itemIconWrapper) },
                  [item.uploading ? (_openBlock(), _createBlock(
                    _component_MkSystemIcon,
                    {
                      key: 0,
                      class: _normalizeClass(_ctx.$style.itemIcon),
                      type: "waiting"
                    },
                    null,
                    2
                    /* CLASS */
                  )) : item.uploaded ? (_openBlock(), _createBlock(
                    _component_MkSystemIcon,
                    {
                      key: 1,
                      class: _normalizeClass(_ctx.$style.itemIcon),
                      type: "success"
                    },
                    null,
                    2
                    /* CLASS */
                  )) : item.uploadFailed ? (_openBlock(), _createBlock(
                    _component_MkSystemIcon,
                    {
                      key: 2,
                      class: _normalizeClass(_ctx.$style.itemIcon),
                      type: "error"
                    },
                    null,
                    2
                    /* CLASS */
                  )) : _createCommentVNode("v-if", true)],
                  2
                  /* CLASS */
                )
              ],
              2
              /* CLASS */
            )], 46, ["onContextmenu"])), [[_directive_panel]]);
          }),
          128
          /* KEYED_FRAGMENT */
        ))],
        2
        /* CLASS */
      );
    };
  }
};
