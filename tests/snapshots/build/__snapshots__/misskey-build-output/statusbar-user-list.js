import { Fragment as _Fragment, Transition as _Transition, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, renderList as _renderList, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue";
import { ref, watch } from "vue";
import { useInterval } from "@@/js/use-interval.js";
import MkMarqueeText from "@/components/MkMarqueeText.vue";
import { misskeyApi } from "@/utility/misskey-api.js";
import { getNoteSummary } from "@/utility/get-note-summary.js";
import { notePage } from "@/filters/note.js";
export default {
  __name: "statusbar-user-list",
  props: {
    userListId: {
      type: String,
      required: false
    },
    display: {
      type: String,
      required: false
    },
    marqueeDuration: {
      type: Number,
      required: false
    },
    marqueeReverse: {
      type: Boolean,
      required: false
    },
    oneByOneInterval: {
      type: Number,
      required: false
    },
    refreshIntervalSec: {
      type: Number,
      required: true
    }
  },
  setup(__props) {
    const props = __props;
    const notes = ref([]);
    const fetching = ref(true);
    const key = ref(0);
    const tick = () => {
      if (props.userListId == null) return;
      misskeyApi("notes/user-list-timeline", { listId: props.userListId }).then((res) => {
        notes.value = res;
        fetching.value = false;
        key.value++;
      });
    };
    watch(() => props.userListId, tick);
    useInterval(tick, Math.max(5e3, props.refreshIntervalSec * 1e3), {
      immediate: true,
      afterMounted: true
    });
    return (_ctx, _cache) => {
      const _component_Mfm = _resolveComponent("Mfm");
      const _component_MkA = _resolveComponent("MkA");
      return !fetching.value ? (_openBlock(), _createElementBlock(
        "span",
        {
          key: 0,
          class: _normalizeClass(_ctx.$style.root)
        },
        [__props.display === "marquee" ? (_openBlock(), _createBlock(_Transition, {
          key: 0,
          enterActiveClass: _ctx.$style.transition_change_enterActive,
          leaveActiveClass: _ctx.$style.transition_change_leaveActive,
          enterFromClass: _ctx.$style.transition_change_enterFrom,
          leaveToClass: _ctx.$style.transition_change_leaveTo,
          mode: "default"
        }, {
          default: _withCtx(() => [_createVNode(MkMarqueeText, {
            key: key.value,
            duration: __props.marqueeDuration,
            reverse: __props.marqueeReverse
          }, {
            default: _withCtx(() => [(_openBlock(true), _createElementBlock(
              _Fragment,
              null,
              _renderList(notes.value, (note) => {
                return _openBlock(), _createElementBlock(
                  "span",
                  {
                    key: note.id,
                    class: _normalizeClass(_ctx.$style.item)
                  },
                  [
                    note.user.avatarUrl ? (_openBlock(), _createElementBlock("img", {
                      key: 0,
                      class: _normalizeClass(_ctx.$style.avatar),
                      src: note.user.avatarUrl,
                      decoding: "async"
                    }, null, 10, ["src"])) : _createCommentVNode("v-if", true),
                    _createVNode(_component_MkA, {
                      class: _normalizeClass(_ctx.$style.text),
                      to: _unref(notePage)(note)
                    }, {
                      default: _withCtx(() => [_createVNode(_component_Mfm, {
                        text: _unref(getNoteSummary)(note),
                        plain: true,
                        nowrap: true
                      }, null, 8, [
                        "text",
                        "plain",
                        "nowrap"
                      ])]),
                      _: 2
                    }, 10, ["to"]),
                    _createElementVNode(
                      "span",
                      { class: _normalizeClass(_ctx.$style.divider) },
                      null,
                      2
                      /* CLASS */
                    )
                  ],
                  2
                  /* CLASS */
                );
              }),
              128
              /* KEYED_FRAGMENT */
            ))]),
            _: 2
          }, 1032, ["duration", "reverse"])]),
          _: 1
        }, 8, [
          "enterActiveClass",
          "leaveActiveClass",
          "enterFromClass",
          "leaveToClass"
        ])) : __props.display === "oneByOne" ? (_openBlock(), _createElementBlock(
          _Fragment,
          { key: 1 },
          null,
          64
          /* STABLE_FRAGMENT */
        )) : _createCommentVNode("v-if", true)],
        2
        /* CLASS */
      )) : _createCommentVNode("v-if", true);
    };
  }
};
