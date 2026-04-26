import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue";
const _hoisted_1 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-search" });
import { ref } from "vue";
import XMessage from "./XMessage.vue";
import MkButton from "@/components/MkButton.vue";
import { i18n } from "@/i18n.js";
import { misskeyApi } from "@/utility/misskey-api.js";
import MkInput from "@/components/MkInput.vue";
import MkFoldableSection from "@/components/MkFoldableSection.vue";
export default {
  __name: "room.search",
  props: {
    userId: {
      type: String,
      required: false
    },
    roomId: {
      type: String,
      required: false
    }
  },
  setup(__props) {
    const props = __props;
    const searchQuery = ref("");
    const searched = ref(false);
    const searchResults = ref([]);
    async function search() {
      const res = await misskeyApi("chat/messages/search", {
        query: searchQuery.value,
        roomId: props.roomId,
        userId: props.userId
      });
      searchResults.value = res;
      searched.value = true;
    }
    return (_ctx, _cache) => {
      const _component_MkResult = _resolveComponent("MkResult");
      return _openBlock(), _createElementBlock("div", { class: "_gaps" }, [
        _createVNode(MkInput, {
          placeholder: _unref(i18n).ts._chat.searchMessages,
          type: "search",
          onEnter: _cache[0] || (_cache[0] = ($event) => search()),
          modelValue: searchQuery.value,
          "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event) => searchQuery.value = $event)
        }, {
          prefix: _withCtx(() => [_hoisted_1]),
          _: 1
        }, 8, ["placeholder", "modelValue"]),
        _createVNode(MkButton, {
          primary: "",
          rounded: "",
          onClick: search
        }, {
          default: _withCtx(() => [_createTextVNode(
            _toDisplayString(_unref(i18n).ts.search),
            1
            /* TEXT */
          )]),
          _: 1
        }),
        searched.value ? (_openBlock(), _createBlock(
          MkFoldableSection,
          { key: 0 },
          {
            header: _withCtx(() => [_createTextVNode(
              _toDisplayString(_unref(i18n).ts.searchResult),
              1
              /* TEXT */
            )]),
            default: _withCtx(() => [searchResults.value.length > 0 ? (_openBlock(), _createElementBlock("div", {
              key: 0,
              class: "_gaps_s"
            }, [(_openBlock(true), _createElementBlock(
              _Fragment,
              null,
              _renderList(searchResults.value, (message) => {
                return _openBlock(), _createElementBlock(
                  "div",
                  {
                    key: message.id,
                    class: _normalizeClass(_ctx.$style.searchResultItem)
                  },
                  [_createVNode(XMessage, {
                    message,
                    user: message.fromUser,
                    isSearchResult: true
                  }, null, 8, [
                    "message",
                    "user",
                    "isSearchResult"
                  ])],
                  2
                  /* CLASS */
                );
              }),
              128
              /* KEYED_FRAGMENT */
            ))])) : (_openBlock(), _createBlock(_component_MkResult, {
              key: 1,
              type: "notFound"
            }))]),
            _: 2
          },
          1024
          /* DYNAMIC_SLOTS */
        )) : _createCommentVNode("v-if", true)
      ]);
    };
  }
};
