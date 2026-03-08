import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("span", { class: "ti ti-dots" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("span", { class: "ti ti-dots" })
import { computed, toRefs } from 'vue'
import MkButton from '@/components/MkButton.vue'
const min = 1;

export default /*@__PURE__*/_defineComponent({
  __name: 'MkPagingButtons',
  props: {
    current: { type: Number, required: true },
    max: { type: Number, required: true },
    buttonCount: { type: Number, required: true }
  },
  emits: ["pageChanged"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
const { current, max } = toRefs(props);
const buttonCount = computed(() => Math.min(max.value, props.buttonCount));
const buttonCountHalf = computed(() => Math.floor(buttonCount.value / 2));
const buttonCountStart = computed(() => Math.min(Math.max(min, current.value - buttonCountHalf.value), max.value - buttonCount.value + 1));
const buttonRanges = computed(() => Array.from({ length: buttonCount.value }, (_, i) => buttonCountStart.value + i));
const prevDotVisible = computed(() => (current.value - 1 > buttonCountHalf.value) && (max.value > buttonCount.value));
const nextDotVisible = computed(() => (current.value < max.value - buttonCountHalf.value) && (max.value > buttonCount.value));
if (_DEV_) {
	console.log('[MkPagingButtons]', current.value, max.value, buttonCount.value, buttonCountHalf.value);
	console.log('[MkPagingButtons]', current.value < max.value - buttonCountHalf.value);
	console.log('[MkPagingButtons]', max.value > buttonCount.value);
}
function onNumberButtonClicked(pageNumber: number) {
	emit('pageChanged', pageNumber);
}
function onToHeadButtonClicked() {
	emit('pageChanged', min);
}
function onToPrevButtonClicked() {
	const newPageNumber = current.value <= min ? min : current.value - 1;
	emit('pageChanged', newPageNumber);
}
function onToNextButtonClicked() {
	const newPageNumber = current.value >= max.value ? max.value : current.value + 1;
	emit('pageChanged', newPageNumber);
}
function onToTailButtonClicked() {
	emit('pageChanged', max.value);
}

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", {
      class: _normalizeClass(_ctx.$style.root)
    }, [ _createVNode(MkButton, {
        primary: "",
        disabled: min === _unref(current),
        onClick: onToPrevButtonClicked
      }, {
        default: _withCtx(() => [
          _createTextVNode("&lt;")
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["disabled"]), _createElementVNode("div", {
        class: _normalizeClass(_ctx.$style.buttons)
      }, [ (prevDotVisible.value) ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            class: _normalizeClass(_ctx.$style.headTailButtons)
          }, [ _createVNode(MkButton, { onClick: onToHeadButtonClicked }, {
              default: _withCtx(() => [
                _createTextVNode(_toDisplayString(min), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }), _hoisted_1 ])) : _createCommentVNode("v-if", true), (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(buttonRanges.value, (i) => {
          return (_openBlock(), _createBlock(MkButton, {
            key: i,
            disabled: _unref(current) === i,
            onClick: _cache[0] || (_cache[0] = ($event: any) => (onNumberButtonClicked(i)))
          }, {
            default: _withCtx(() => [
              _createTextVNode(_toDisplayString(i), 1 /* TEXT */)
            ]),
            _: 2 /* DYNAMIC */
          }, 1032 /* PROPS, DYNAMIC_SLOTS */, ["disabled"]))
        }), 128 /* KEYED_FRAGMENT */)), (nextDotVisible.value) ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            class: _normalizeClass(_ctx.$style.headTailButtons)
          }, [ _hoisted_2, _createVNode(MkButton, { onClick: onToTailButtonClicked }, {
              default: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(max)), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }) ])) : _createCommentVNode("v-if", true) ]), _createVNode(MkButton, {
        primary: "",
        disabled: _unref(max) === _unref(current),
        onClick: onToNextButtonClicked
      }, {
        default: _withCtx(() => [
          _createTextVNode("&gt;")
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["disabled"]) ]))
}
}

})
