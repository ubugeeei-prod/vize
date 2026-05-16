import { Fragment as _Fragment, Transition as _Transition, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx } from "vue";
import { ref } from "vue";
import { useInterval } from "@@/js/use-interval.js";
import MkMarqueeText from "@/components/MkMarqueeText.vue";
import { shuffle } from "@/utility/shuffle.js";
export default {
  __name: "statusbar-rss",
  props: {
    url: {
      type: String,
      required: true
    },
    shuffle: {
      type: Boolean,
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
    const items = ref([]);
    const fetching = ref(true);
    const key = ref(0);
    const tick = () => {
      window.fetch(`/api/fetch-rss?url=${encodeURIComponent(props.url)}`, {}).then((res) => {
        res.json().then((feed) => {
          if (props.shuffle) {
            shuffle(feed.items);
          }
          items.value = feed.items;
          fetching.value = false;
          key.value++;
        });
      });
    };
    useInterval(tick, Math.max(5e3, props.refreshIntervalSec * 1e3), {
      immediate: true,
      afterMounted: true
    });
    return (_ctx, _cache) => {
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
              _renderList(items.value, (item) => {
                return _openBlock(), _createElementBlock(
                  "span",
                  { class: _normalizeClass(_ctx.$style.item) },
                  [_createElementVNode("a", {
                    href: item.link,
                    rel: "nofollow noopener",
                    target: "_blank",
                    title: item.title
                  }, _toDisplayString(item.title), 9, ["href", "title"]), _createElementVNode(
                    "span",
                    { class: _normalizeClass(_ctx.$style.divider) },
                    null,
                    2
                    /* CLASS */
                  )],
                  2
                  /* CLASS */
                );
              }),
              256
              /* UNKEYED_FRAGMENT */
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
