import { openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, unref as _unref } from "vue";
import { computed, nextTick, onMounted, onUnmounted, ref, toRefs, useTemplateRef, watch } from "vue";
export default {
  __name: "MkHeaderCell",
  props: {
    column: {
      type: null,
      required: true
    },
    bus: {
      type: null,
      required: true
    }
  },
  emits: [
    "operation:beginWidthChange",
    "operation:endWidthChange",
    "operation:widthLargest",
    "change:width",
    "change:contentSize"
  ],
  setup(__props, { emit: __emit }) {
    const emit = __emit;
    const props = __props;
    const { column, bus } = toRefs(props);
    const rootEl = useTemplateRef("rootEl");
    const contentEl = useTemplateRef("contentEl");
    const resizing = ref(false);
    const text = computed(() => {
      const result = column.value.setting.title ?? column.value.setting.bindTo;
      return result.length > 0 ? result : "　";
    });
    watch(column, () => {
      // 中身がセットされた直後はサイズが分からないので、次のタイミングで更新する
      nextTick(emitContentSizeChanged);
    }, { immediate: true });
    function onHandleDoubleClick(ev) {
      switch (ev.type) {
        case "dblclick": {
          emit("operation:widthLargest", column.value);
          break;
        }
      }
    }
    function onHandleMouseDown(ev) {
      switch (ev.type) {
        case "mousedown": {
          if (!resizing.value) {
            registerHandleMouseUp();
            registerHandleMouseMove();
            resizing.value = true;
            emit("operation:beginWidthChange", column.value);
          }
          break;
        }
      }
    }
    function onHandleMouseMove(ev) {
      if (!rootEl.value) {
        // 型ガード
        return;
      }
      switch (ev.type) {
        case "mousemove": {
          if (resizing.value) {
            const bounds = rootEl.value.getBoundingClientRect();
            const clientWidth = rootEl.value.clientWidth;
            const clientRight = bounds.left + clientWidth;
            const nextWidth = clientWidth + (ev.clientX - clientRight);
            emit("change:width", column.value, `${nextWidth}px`);
          }
          break;
        }
      }
    }
    function onHandleMouseUp(ev) {
      switch (ev.type) {
        case "mouseup": {
          if (resizing.value) {
            unregisterHandleMouseUp();
            unregisterHandleMouseMove();
            resizing.value = false;
            emit("operation:endWidthChange", column.value);
          }
          break;
        }
      }
    }
    function onForceRefreshContentSize() {
      emitContentSizeChanged();
    }
    function registerHandleMouseMove() {
      unregisterHandleMouseMove();
      addEventListener("mousemove", onHandleMouseMove);
    }
    function unregisterHandleMouseMove() {
      removeEventListener("mousemove", onHandleMouseMove);
    }
    function registerHandleMouseUp() {
      unregisterHandleMouseUp();
      addEventListener("mouseup", onHandleMouseUp);
    }
    function unregisterHandleMouseUp() {
      removeEventListener("mouseup", onHandleMouseUp);
    }
    function emitContentSizeChanged() {
      const clientWidth = contentEl.value?.clientWidth ?? 0;
      const clientHeight = contentEl.value?.clientHeight ?? 0;
      emit("change:contentSize", column.value, {
        // バーの横幅も考慮したいので、+3px
        width: clientWidth + 3 + 3,
        height: clientHeight
      });
    }
    onMounted(() => {
      bus.value.on("forceRefreshContentSize", onForceRefreshContentSize);
    });
    onUnmounted(() => {
      bus.value.off("forceRefreshContentSize", onForceRefreshContentSize);
    });
    return (_ctx, _cache) => {
      return _openBlock(), _createElementBlock("div", {
        ref_key: "rootEl",
        ref: rootEl,
        class: _normalizeClass(["mk_grid_th", _ctx.$style.cell]),
        style: _normalizeStyle([{
          maxWidth: _unref(column).width,
          minWidth: _unref(column).width,
          width: _unref(column).width
        }]),
        "data-grid-cell": "",
        "data-grid-cell-row": -1,
        "data-grid-cell-col": _unref(column).index
      }, [_createElementVNode(
        "div",
        { class: _normalizeClass(_ctx.$style.root) },
        [
          _createElementVNode(
            "div",
            { class: _normalizeClass(_ctx.$style.left) },
            null,
            2
            /* CLASS */
          ),
          _createElementVNode(
            "div",
            { class: _normalizeClass(_ctx.$style.wrapper) },
            [_createElementVNode(
              "div",
              {
                ref_key: "contentEl",
                ref: contentEl,
                class: _normalizeClass(_ctx.$style.contentArea)
              },
              [_unref(column).setting.icon ? (_openBlock(), _createElementBlock(
                "span",
                {
                  key: 0,
                  class: _normalizeClass(["ti", _unref(column).setting.icon]),
                  style: "line-height: normal"
                },
                null,
                2
                /* CLASS */
              )) : (_openBlock(), _createElementBlock(
                "span",
                { key: 1 },
                _toDisplayString(text.value),
                1
                /* TEXT */
              ))],
              2
              /* CLASS */
            )],
            2
            /* CLASS */
          ),
          _createElementVNode(
            "div",
            {
              class: _normalizeClass(_ctx.$style.right),
              onMousedown: onHandleMouseDown,
              onDblclick: onHandleDoubleClick
            },
            null,
            34
            /* CLASS, NEED_HYDRATION */
          )
        ],
        2
        /* CLASS */
      )], 14, ["data-grid-cell-row", "data-grid-cell-col"]);
    };
  }
};
