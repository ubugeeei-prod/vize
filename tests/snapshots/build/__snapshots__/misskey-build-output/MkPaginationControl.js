import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveDirective as _resolveDirective, withDirectives as _withDirectives, renderSlot as _renderSlot, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue";
const _hoisted_1 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-arrows-sort" });
const _hoisted_2 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-search" });
const _hoisted_3 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-filter" });
const _hoisted_4 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-calendar-clock" });
const _hoisted_5 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-refresh" });
const _hoisted_6 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-search" });
import { ref, watch } from "vue";
import MkButton from "@/components/MkButton.vue";
import { i18n } from "@/i18n.js";
import MkSelect from "@/components/MkSelect.vue";
import MkInput from "@/components/MkInput.vue";
import { formatDateTimeString } from "@/utility/format-time-string.js";
import { useMkSelect } from "@/composables/use-mkselect.js";
export default {
  __name: "MkPaginationControl",
  props: {
    paginator: {
      type: null,
      required: true
    },
    canFilter: {
      type: Boolean,
      required: false,
      default: false
    },
    filterOpened: {
      type: Boolean,
      required: false,
      default: false
    }
  },
  setup(__props) {
    const props = __props;
    const searchOpened = ref(false);
    const filterOpened = ref(props.filterOpened);
    const { model: order, def: orderDef } = useMkSelect({
      items: [{
        label: i18n.ts._order.newest,
        value: "newest"
      }, {
        label: i18n.ts._order.oldest,
        value: "oldest"
      }],
      initialValue: "newest"
    });
    const date = ref(null);
    const q = ref(null);
    watch(order, () => {
      props.paginator.order.value = order.value;
      props.paginator.initialDirection = order.value === "oldest" ? "newer" : "older";
      props.paginator.reload();
    });
    watch(date, () => {
      props.paginator.initialDate = date.value;
      props.paginator.reload();
    });
    watch(q, () => {
      props.paginator.searchQuery.value = q.value;
      props.paginator.reload();
    });
    return (_ctx, _cache) => {
      const _directive_tooltip = _resolveDirective("tooltip");
      return _openBlock(), _createElementBlock(
        "div",
        { class: _normalizeClass(_ctx.$style.root) },
        [
          _createElementVNode(
            "div",
            { class: _normalizeClass(_ctx.$style.control) },
            [
              _createVNode(MkSelect, {
                class: _normalizeClass(_ctx.$style.order),
                items: _unref(orderDef),
                modelValue: _unref(order),
                "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event) => order.value = $event)
              }, {
                prefix: _withCtx(() => [_hoisted_1]),
                _: 1
              }, 10, ["items", "modelValue"]),
              __props.paginator.canSearch ? _withDirectives((_openBlock(), _createBlock(MkButton, {
                key: 0,
                iconOnly: "",
                transparent: "",
                rounded: "",
                active: searchOpened.value,
                onClick: _cache[1] || (_cache[1] = ($event) => searchOpened.value = !searchOpened.value)
              }, {
                default: _withCtx(() => [_hoisted_2]),
                _: 1
              }, 8, ["active"])), [[_directive_tooltip, _unref(i18n).ts.search]]) : _createCommentVNode("v-if", true),
              __props.canFilter ? _withDirectives((_openBlock(), _createBlock(MkButton, {
                key: 0,
                iconOnly: "",
                transparent: "",
                rounded: "",
                active: filterOpened.value,
                onClick: _cache[2] || (_cache[2] = ($event) => filterOpened.value = !filterOpened.value)
              }, {
                default: _withCtx(() => [_hoisted_3]),
                _: 1
              }, 8, ["active"])), [[_directive_tooltip, _unref(i18n).ts.filter]]) : _createCommentVNode("v-if", true),
              _withDirectives(_createVNode(MkButton, {
                iconOnly: "",
                transparent: "",
                rounded: "",
                active: date.value != null,
                onClick: _cache[3] || (_cache[3] = ($event) => date.value = date.value == null ? Date.now() : null)
              }, {
                default: _withCtx(() => [_hoisted_4]),
                _: 1
              }, 8, ["active"]), [[_directive_tooltip, _unref(i18n).ts.dateAndTime]]),
              _withDirectives(_createVNode(MkButton, {
                iconOnly: "",
                transparent: "",
                rounded: "",
                onClick: _cache[4] || (_cache[4] = ($event) => __props.paginator.reload())
              }, {
                default: _withCtx(() => [_hoisted_5]),
                _: 1
              }), [[_directive_tooltip, _unref(i18n).ts.reload]])
            ],
            2
            /* CLASS */
          ),
          searchOpened.value ? (_openBlock(), _createBlock(MkInput, {
            key: 0,
            type: "search",
            debounce: "",
            modelValue: q.value,
            "onUpdate:modelValue": _cache[5] || (_cache[5] = ($event) => q.value = $event)
          }, {
            label: _withCtx(() => [_createTextVNode(
              _toDisplayString(_unref(i18n).ts.search),
              1
              /* TEXT */
            )]),
            prefix: _withCtx(() => [_hoisted_6]),
            _: 1
          }, 8, ["modelValue"])) : _createCommentVNode("v-if", true),
          date.value != null ? (_openBlock(), _createBlock(MkInput, {
            key: 0,
            type: "date",
            modelValue: _unref(formatDateTimeString)(new Date(date.value), "yyyy-MM-dd"),
            "onUpdate:modelValue": _cache[6] || (_cache[6] = ($event) => date.value = new Date($event).getTime())
          }, null, 8, ["modelValue"])) : _createCommentVNode("v-if", true),
          filterOpened.value ? _renderSlot(_ctx.$slots, "default", { key: 0 }) : _createCommentVNode("v-if", true)
        ],
        2
        /* CLASS */
      );
    };
  }
};
