import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue";
const _hoisted_1 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-chevron-up" });
const _hoisted_2 = /* @__PURE__ */ _createElementVNode("span", { style: "height: 1em; width: 1px; background: var(--MI_THEME-divider);" });
const _hoisted_3 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-chevron-down" });
import MkNote from "@/components/MkNote.vue";
import MkPagination from "@/components/MkPagination.vue";
import { i18n } from "@/i18n.js";
import { useGlobalEvent } from "@/events.js";
import { isSeparatorNeeded, getSeparatorInfo } from "@/utility/timeline-date-separate.js";
export default {
  __name: "MkNotesTimeline",
  props: {
    paginator: {
      type: null,
      required: true
    },
    noGap: {
      type: Boolean,
      required: false
    }
  },
  setup(__props, { expose: __expose }) {
    const props = __props;
    useGlobalEvent("noteDeleted", (noteId) => {
      props.paginator.removeItem(noteId);
    });
    function reload() {
      return props.paginator.reload();
    }
    __expose({ reload });
    return (_ctx, _cache) => {
      const _component_MkResult = _resolveComponent("MkResult");
      const _component_MkAd = _resolveComponent("MkAd");
      return _openBlock(), _createBlock(MkPagination, {
        paginator: __props.paginator,
        direction: _ctx.direction,
        autoLoad: _ctx.autoLoad,
        pullToRefresh: _ctx.pullToRefresh,
        withControl: _ctx.withControl,
        forceDisableInfiniteScroll: _ctx.forceDisableInfiniteScroll
      }, {
        empty: _withCtx(() => [_createVNode(_component_MkResult, {
          type: "empty",
          text: _unref(i18n).ts.noNotes
        }, null, 8, ["text"])]),
        default: _withCtx(({ items: notes }) => [_createElementVNode(
          "div",
          { class: _normalizeClass([_ctx.$style.root, {
            [_ctx.$style.noGap]: __props.noGap,
            "_gaps": !__props.noGap
          }]) },
          [(_openBlock(true), _createElementBlock(
            _Fragment,
            null,
            _renderList(notes, (note, i) => {
              return _openBlock(), _createElementBlock(
                _Fragment,
                { key: note.id },
                [i > 0 && _unref(isSeparatorNeeded)(__props.paginator.items.value[i - 1].createdAt, note.createdAt) ? (_openBlock(), _createElementBlock("div", {
                  key: 0,
                  "data-scroll-anchor": note.id,
                  class: _normalizeClass({ "_gaps": !__props.noGap })
                }, [
                  _createElementVNode(
                    "div",
                    { class: _normalizeClass([_ctx.$style.date, { [_ctx.$style.noGap]: __props.noGap }]) },
                    [
                      _createElementVNode("span", null, [_hoisted_1, _createTextVNode(
                        " " + _toDisplayString(_unref(getSeparatorInfo)(__props.paginator.items.value[i - 1].createdAt, note.createdAt)?.prevText),
                        1
                        /* TEXT */
                      )]),
                      _hoisted_2,
                      _createElementVNode("span", null, [_createTextVNode(
                        _toDisplayString(_unref(getSeparatorInfo)(__props.paginator.items.value[i - 1].createdAt, note.createdAt)?.nextText) + " ",
                        1
                        /* TEXT */
                      ), _hoisted_3])
                    ],
                    2
                    /* CLASS */
                  ),
                  _createVNode(MkNote, {
                    class: _normalizeClass(_ctx.$style.note),
                    note,
                    withHardMute: true
                  }, null, 10, ["note", "withHardMute"]),
                  note._shouldInsertAd_ ? (_openBlock(), _createElementBlock(
                    "div",
                    {
                      key: 0,
                      class: _normalizeClass(_ctx.$style.ad)
                    },
                    [_createVNode(_component_MkAd, { preferForms: ["horizontal", "horizontal-big"] }, null, 8, ["preferForms"])],
                    2
                    /* CLASS */
                  )) : _createCommentVNode("v-if", true)
                ], 10, ["data-scroll-anchor"])) : note._shouldInsertAd_ ? (_openBlock(), _createElementBlock("div", {
                  key: 1,
                  class: _normalizeClass({ "_gaps": !__props.noGap }),
                  "data-scroll-anchor": note.id
                }, [_createVNode(MkNote, {
                  class: _normalizeClass(_ctx.$style.note),
                  note,
                  withHardMute: true
                }, null, 10, ["note", "withHardMute"]), _createElementVNode(
                  "div",
                  { class: _normalizeClass(_ctx.$style.ad) },
                  [_createVNode(_component_MkAd, { preferForms: ["horizontal", "horizontal-big"] }, null, 8, ["preferForms"])],
                  2
                  /* CLASS */
                )], 10, ["data-scroll-anchor"])) : (_openBlock(), _createBlock(MkNote, {
                  key: 2,
                  class: _normalizeClass(_ctx.$style.note),
                  note,
                  withHardMute: true,
                  "data-scroll-anchor": note.id
                }, null, 10, [
                  "note",
                  "withHardMute",
                  "data-scroll-anchor"
                ]))],
                64
                /* STABLE_FRAGMENT */
              );
            }),
            128
            /* KEYED_FRAGMENT */
          ))],
          2
          /* CLASS */
        )]),
        _: 1
      }, 8, [
        "paginator",
        "direction",
        "autoLoad",
        "pullToRefresh",
        "withControl",
        "forceDisableInfiniteScroll"
      ]);
    };
  }
};
