import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createCommentVNode as _createCommentVNode, resolveDynamicComponent as _resolveDynamicComponent, renderList as _renderList, renderSlot as _renderSlot, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue";
import { inject, watch, ref } from "vue";
import { TransitionGroup } from "vue";
import { isSupportedEmoji } from "@@/js/emojilist.js";
import XReaction from "@/components/MkReactionsViewer.reaction.vue";
import { $i } from "@/i.js";
import { prefer } from "@/preferences.js";
import { customEmojisMap } from "@/custom-emojis.js";
import { DI } from "@/di.js";
export default {
  __name: "MkReactionsViewer",
  props: {
    noteId: {
      type: null,
      required: true
    },
    reactions: {
      type: null,
      required: true
    },
    reactionEmojis: {
      type: null,
      required: true
    },
    myReaction: {
      type: null,
      required: true
    },
    maxNumber: {
      type: Number,
      required: false,
      default: Infinity
    }
  },
  emits: ["mockUpdateMyReaction"],
  setup(__props, { emit: __emit }) {
    const emit = __emit;
    const props = __props;
    const mock = inject(DI.mock, false);
    const initialReactions = new Set(Object.keys(props.reactions));
    const _reactions = ref([]);
    const hasMoreReactions = ref(false);
    if (props.myReaction != null && !(props.myReaction in props.reactions)) {
      _reactions.value.push([props.myReaction, props.reactions[props.myReaction]]);
    }
    function onMockToggleReaction(emoji, count) {
      if (!mock) return;
      const i = _reactions.value.findIndex((item) => item[0] === emoji);
      if (i < 0) return;
      emit("mockUpdateMyReaction", emoji, count - _reactions.value[i][1]);
    }
    function canReact(reaction) {
      if (!$i) return false;
      // TODO: CheckPermissions
      return !reaction.match(/@\w/) && (customEmojisMap.has(reaction) || isSupportedEmoji(reaction));
    }
    watch([() => props.reactions, () => props.maxNumber], ([newSource, maxNumber]) => {
      let newReactions = [];
      hasMoreReactions.value = Object.keys(newSource).length > maxNumber;
      for (let i = 0; i < _reactions.value.length; i++) {
        const reaction = _reactions.value[i][0];
        if (reaction in newSource && newSource[reaction] !== 0) {
          _reactions.value[i][1] = newSource[reaction];
          newReactions.push(_reactions.value[i]);
        }
      }
      const newReactionsNames = newReactions.map(([x]) => x);
      newReactions = [...newReactions, ...Object.entries(newSource).sort(([emojiA, countA], [emojiB, countB]) => {
        if (prefer.s.showAvailableReactionsFirstInNote) {
          if (!canReact(emojiA) && canReact(emojiB)) return 1;
          if (canReact(emojiA) && !canReact(emojiB)) return -1;
          return countB - countA;
        } else {
          return countB - countA;
        }
      }).filter(([y], i) => i < maxNumber && !newReactionsNames.includes(y))];
      newReactions = newReactions.slice(0, props.maxNumber);
      if (props.myReaction && !newReactions.map(([x]) => x).includes(props.myReaction)) {
        newReactions.push([props.myReaction, newSource[props.myReaction]]);
      }
      _reactions.value = newReactions;
    }, {
      immediate: true,
      deep: true
    });
    return (_ctx, _cache) => {
      return _openBlock(), _createBlock(_resolveDynamicComponent(_unref(prefer).s.animation ? _unref(TransitionGroup) : "div"), {
        enterActiveClass: _ctx.$style.transition_x_enterActive,
        leaveActiveClass: _ctx.$style.transition_x_leaveActive,
        enterFromClass: _ctx.$style.transition_x_enterFrom,
        leaveToClass: _ctx.$style.transition_x_leaveTo,
        moveClass: _ctx.$style.transition_x_move,
        tag: "div",
        class: _normalizeClass(_ctx.$style.root)
      }, {
        default: _withCtx(() => [(_openBlock(true), _createElementBlock(
          _Fragment,
          null,
          _renderList(_reactions.value, ([reaction, count]) => {
            return _openBlock(), _createBlock(XReaction, {
              key: reaction,
              reaction,
              reactionEmojis: props.reactionEmojis,
              count,
              isInitial: _unref(initialReactions).has(reaction),
              noteId: props.noteId,
              myReaction: props.myReaction,
              onReactionToggled: onMockToggleReaction
            }, null, 8, [
              "reaction",
              "reactionEmojis",
              "count",
              "isInitial",
              "noteId",
              "myReaction"
            ]);
          }),
          128
          /* KEYED_FRAGMENT */
        )), hasMoreReactions.value ? _renderSlot(_ctx.$slots, "more", { key: 0 }) : _createCommentVNode("v-if", true)]),
        _: 3
      }, 1034, [
        "enterActiveClass",
        "leaveActiveClass",
        "enterFromClass",
        "leaveToClass",
        "moveClass"
      ]);
    };
  }
};
