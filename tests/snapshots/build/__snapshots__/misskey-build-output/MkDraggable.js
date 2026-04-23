import { useSlots as _useSlots } from "vue";
import { Fragment as _Fragment, TransitionGroup as _TransitionGroup, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, renderList as _renderList, renderSlot as _renderSlot, normalizeClass as _normalizeClass, withCtx as _withCtx, withModifiers as _withModifiers } from "vue";
import { nextTick } from "vue";
import { getDragData, setDragData } from "@/drag-and-drop.js";
import { genId } from "@/utility/id.js";
import { ref } from "vue";
// 別々のコンポーネントインスタンス間でD&Dを融通するためにグローバルに状態を持っておく必要がある
const dragging = ref(false);
let dropCallback = null;
export default {
  __name: "MkDraggable",
  props: {
    modelValue: {
      type: Array,
      required: true
    },
    direction: {
      type: String,
      required: true
    },
    group: {
      type: [String, null],
      required: false,
      default: null
    },
    manualDragStart: {
      type: Boolean,
      required: false,
      default: false
    },
    withGaps: {
      type: Boolean,
      required: false,
      default: false
    },
    canNest: {
      type: Boolean,
      required: false,
      default: false
    }
  },
  emits: ["update:modelValue"],
  setup(__props, { emit: __emit }) {
    const emit = __emit;
    const props = __props;
    const slots = _useSlots();
    const dropReadyArea = ref([null, null]);
    const instanceId = genId();
    const group = props.group ?? instanceId;
    function onDragstart(ev, item) {
      if (ev.dataTransfer == null) return;
      ev.dataTransfer.effectAllowed = "move";
      setDragData(ev, "MkDraggable", {
        item,
        instanceId,
        group
      });
      const target = ev.target;
      target.addEventListener("dragend", (ev) => {
        dragging.value = false;
        dropReadyArea.value = [null, null];
      }, { once: true });
      dropCallback = (targetInstanceId) => {
        if (targetInstanceId === instanceId) return;
        const newValue = props.modelValue.filter((x) => x.id !== item.id);
        emit("update:modelValue", newValue);
      };
      // Chromeのバグで、Dragstartハンドラ内ですぐにDOMを変更する(=リアクティブなプロパティを変更する)とDragが終了してしまう
      // SEE: https://stackoverflow.com/questions/19639969/html5-dragend-event-firing-immediately
      window.setTimeout(() => {
        dragging.value = true;
      }, 10);
    }
    function onDragover(ev, item, backward) {
      nextTick(() => {
        dropReadyArea.value = [item.id, backward ? "backward" : "forward"];
      });
    }
    function onDragleave(ev, item) {
      dropReadyArea.value = [null, null];
    }
    function onDrop(ev, item, backward) {
      const dragged = getDragData(ev, "MkDraggable");
      dropReadyArea.value = [null, null];
      if (dragged == null || dragged.group !== group || dragged.item.id === item.id) return;
      dropCallback?.(instanceId);
      const fromIndex = props.modelValue.findIndex((x) => x.id === dragged.item.id);
      let toIndex = props.modelValue.findIndex((x) => x.id === item.id);
      const newValue = [...props.modelValue];
      if (fromIndex > -1) newValue.splice(fromIndex, 1);
      toIndex = newValue.findIndex((x) => x.id === item.id);
      if (backward) toIndex += 1;
      newValue.splice(toIndex, 0, dragged.item);
      emit("update:modelValue", newValue);
    }
    function onEmptyDrop(ev) {
      const dragged = getDragData(ev, "MkDraggable");
      if (dragged == null) return;
      dropCallback?.(instanceId);
      emit("update:modelValue", [dragged.item]);
    }
    return (_ctx, _cache) => {
      return _openBlock(), _createBlock(_TransitionGroup, {
        tag: "div",
        enterActiveClass: _ctx.$style.transition_items_enterActive,
        leaveActiveClass: _ctx.$style.transition_items_leaveActive,
        enterFromClass: _ctx.$style.transition_items_enterFrom,
        leaveToClass: _ctx.$style.transition_items_leaveTo,
        moveClass: _ctx.$style.transition_items_move,
        class: _normalizeClass([_ctx.$style.items, {
          [_ctx.$style.dragging]: _ctx.dragging,
          [_ctx.$style.horizontal]: __props.direction === "horizontal",
          [_ctx.$style.vertical]: __props.direction === "vertical",
          [_ctx.$style.withGaps]: __props.withGaps,
          [_ctx.$style.canNest]: __props.canNest
        }])
      }, {
        default: _withCtx(() => [
          _renderSlot(_ctx.$slots, "header"),
          __props.modelValue.length === 0 ? (_openBlock(), _createElementBlock(
            "div",
            {
              key: 0,
              class: _normalizeClass(_ctx.$style.emptyDropArea),
              onDragover: _cache[0] || (_cache[0] = _withModifiers(() => {}, ["prevent", "stop"])),
              onDragleave: _cache[1] || (_cache[1] = () => {}),
              onDrop: _cache[2] || (_cache[2] = _withModifiers(($event) => onEmptyDrop($event), ["prevent", "stop"]))
            },
            null,
            34
            /* CLASS, NEED_HYDRATION */
          )) : _createCommentVNode("v-if", true),
          (_openBlock(true), _createElementBlock(
            _Fragment,
            null,
            _renderList(__props.modelValue, (item, i) => {
              return _openBlock(), _createElementBlock("div", {
                key: `MkDraggableRoot:${item.id}`,
                class: _normalizeClass(_ctx.$style.item),
                draggable: !__props.manualDragStart,
                onDragstart: _withModifiers(($event) => onDragstart($event, item), ["stop"])
              }, [
                _createElementVNode("div", {
                  class: _normalizeClass([_ctx.$style.forwardArea, { [_ctx.$style.dropReady]: dropReadyArea.value[0] === item.id && dropReadyArea.value[1] === "forward" }]),
                  onDragover: _withModifiers(($event) => onDragover($event, item, false), ["prevent", "stop"]),
                  onDragleave: ($event) => onDragleave($event, item),
                  onDrop: _withModifiers(($event) => onDrop($event, item, false), ["prevent", "stop"])
                }, null, 42, [
                  "onDragover",
                  "onDragleave",
                  "onDrop"
                ]),
                _createElementVNode("div", {
                  key: `MkDraggableItem:${item.id}`,
                  style: "position: relative; z-index: 0;"
                }, [_renderSlot(_ctx.$slots, "default", {
                  item,
                  index: i,
                  dragStart: (ev) => onDragstart(ev, item)
                })]),
                _createElementVNode("div", {
                  class: _normalizeClass([_ctx.$style.backwardArea, { [_ctx.$style.dropReady]: dropReadyArea.value[0] === item.id && dropReadyArea.value[1] === "backward" }]),
                  onDragover: _withModifiers(($event) => onDragover($event, item, true), ["prevent", "stop"]),
                  onDragleave: ($event) => onDragleave($event, item),
                  onDrop: _withModifiers(($event) => onDrop($event, item, true), ["prevent", "stop"])
                }, null, 42, [
                  "onDragover",
                  "onDragleave",
                  "onDrop"
                ])
              ], 42, ["draggable", "onDragstart"]);
            }),
            128
            /* KEYED_FRAGMENT */
          )),
          _renderSlot(_ctx.$slots, "footer")
        ]),
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
