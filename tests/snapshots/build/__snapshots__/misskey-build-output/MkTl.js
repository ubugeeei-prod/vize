import { Fragment as _Fragment, openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, renderList as _renderList, renderSlot as _renderSlot, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass } from "vue";
const _hoisted_1 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-chevron-up" });
import { computed } from "vue";
export {};
export default {
  __name: "MkTl",
  props: {
    events: {
      type: Array,
      required: true
    },
    groupBy: {
      type: String,
      required: false,
      default: "d"
    }
  },
  setup(__props) {
    const props = __props;
    const events = computed(() => {
      return props.events.toSorted((a, b) => b.timestamp - a.timestamp);
    });
    function getDateText(dateInstance) {
      const year = dateInstance.getFullYear();
      const month = dateInstance.getMonth() + 1;
      const date = dateInstance.getDate();
      const hour = dateInstance.getHours();
      return `${year.toString()}/${month.toString()}/${date.toString()} ${hour.toString().padStart(2, "0")}:00:00`;
    }
    const items = computed(() => {
      const results = [];
      for (let i = 0; i < events.value.length; i++) {
        const item = events.value[i];
        const date = new Date(item.timestamp);
        const nextDate = events.value[i + 1] ? new Date(events.value[i + 1].timestamp) : null;
        results.push({
          id: item.id,
          type: "event",
          timestamp: item.timestamp,
          delta: i === events.value.length - 1 ? 0 : item.timestamp - events.value[i + 1].timestamp,
          data: item.data
        });
        if (i !== events.value.length - 1 && nextDate != null) {
          let needsSeparator = false;
          if (props.groupBy === "d") {
            needsSeparator = date.getFullYear() !== nextDate.getFullYear() || date.getMonth() !== nextDate.getMonth() || date.getDate() !== nextDate.getDate();
          } else if (props.groupBy === "h") {
            needsSeparator = date.getFullYear() !== nextDate.getFullYear() || date.getMonth() !== nextDate.getMonth() || date.getDate() !== nextDate.getDate() || date.getHours() !== nextDate.getHours();
          }
          if (needsSeparator) {
            results.push({
              id: `date-${item.id}`,
              type: "date",
              prev: date,
              prevText: getDateText(date),
              next: nextDate,
              nextText: getDateText(nextDate)
            });
          }
        }
      }
      return results;
    });
    return (_ctx, _cache) => {
      return _openBlock(), _createElementBlock(
        "div",
        { class: _normalizeClass(_ctx.$style.items) },
        [(_openBlock(true), _createElementBlock(
          _Fragment,
          null,
          _renderList(items.value, (item, i) => {
            return _openBlock(), _createElementBlock(
              _Fragment,
              { key: item.id },
              [
                _createElementVNode(
                  "div",
                  { class: _normalizeClass(_ctx.$style.left) },
                  [item.type === "event" ? _renderSlot(_ctx.$slots, "left", {
                    key: 0,
                    event: item.data,
                    timestamp: item.timestamp,
                    delta: item.delta
                  }) : _createCommentVNode("v-if", true)],
                  2
                  /* CLASS */
                ),
                _createElementVNode(
                  "div",
                  { class: _normalizeClass([
                    _ctx.$style.center,
                    item.type === "date" ? _ctx.$style.date : "",
                    i === 0 ? _ctx.$style.first : "",
                    i === items.value.length - 1 ? _ctx.$style.last : ""
                  ]) },
                  [_createElementVNode(
                    "div",
                    { class: _normalizeClass(_ctx.$style.centerLine) },
                    null,
                    2
                    /* CLASS */
                  ), _createElementVNode(
                    "div",
                    { class: _normalizeClass(_ctx.$style.centerPoint) },
                    null,
                    2
                    /* CLASS */
                  )],
                  2
                  /* CLASS */
                ),
                _createElementVNode(
                  "div",
                  { class: _normalizeClass(_ctx.$style.right) },
                  [item.type === "event" ? _renderSlot(_ctx.$slots, "right", {
                    key: 0,
                    event: item.data,
                    timestamp: item.timestamp,
                    delta: item.delta
                  }) : (_openBlock(), _createElementBlock(
                    "div",
                    {
                      key: 1,
                      class: _normalizeClass(_ctx.$style.dateLabel)
                    },
                    [_hoisted_1, _createTextVNode(
                      " " + _toDisplayString(item.prevText),
                      1
                      /* TEXT */
                    )],
                    2
                    /* CLASS */
                  ))],
                  2
                  /* CLASS */
                )
              ],
              64
              /* STABLE_FRAGMENT */
            );
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
