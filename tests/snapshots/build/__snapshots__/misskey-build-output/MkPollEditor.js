import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, renderList as _renderList, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-alert-triangle" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-x" })
import { ref, watch } from 'vue'
import MkInput from './MkInput.vue'
import MkSelect from './MkSelect.vue'
import MkSwitch from './MkSwitch.vue'
import MkButton from './MkButton.vue'
import { formatDateTimeString } from '@/utility/format-time-string.js'
import { addTime } from '@/utility/time.js'
import { i18n } from '@/i18n.js'
import { useMkSelect } from '@/composables/use-mkselect.js'

export type PollEditorModelValue = {
	expiresAt: number | null;
	expiredAfter: number | null;
	choices: string[];
	multiple: boolean;
};

export default /*@__PURE__*/_defineComponent({
  __name: 'MkPollEditor',
  props: {
    modelValue: { type: Object, required: true }
  },
  emits: ["update:modelValue"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
const choices = ref(props.modelValue.choices);
const multiple = ref(props.modelValue.multiple);
const {
	model: expiration,
	def: expirationDef,
} = useMkSelect({
	items: [
		{ label: i18n.ts._poll.infinite, value: 'infinite' },
		{ label: i18n.ts._poll.at, value: 'at' },
		{ label: i18n.ts._poll.after, value: 'after' },
	],
	initialValue: 'infinite',
});
const atDate = ref(formatDateTimeString(addTime(new Date(), 1, 'day'), 'yyyy-MM-dd'));
const atTime = ref('00:00');
const after = ref(0);
const {
	model: unit,
	def: unitDef,
} = useMkSelect({
	items: [
		{ label: i18n.ts._time.second, value: 'second' },
		{ label: i18n.ts._time.minute, value: 'minute' },
		{ label: i18n.ts._time.hour, value: 'hour' },
		{ label: i18n.ts._time.day, value: 'day' },
	],
	initialValue: 'second',
});
if (props.modelValue.expiresAt) {
	expiration.value = 'at';
	const expiresAt = new Date(props.modelValue.expiresAt);
	atDate.value = formatDateTimeString(expiresAt, 'yyyy-MM-dd');
	atTime.value = formatDateTimeString(expiresAt, 'HH:mm');
} else if (typeof props.modelValue.expiredAfter === 'number') {
	expiration.value = 'after';
	after.value = props.modelValue.expiredAfter / 1000;
} else {
	expiration.value = 'infinite';
}
function onInput(i: number, value: string) {
	choices.value[i] = value;
}
function add() {
	choices.value.push('');
	// TODO
	// nextTick(() => {
	//   (this.$refs.choices as any).childNodes[this.choices.length - 1].childNodes[0].focus();
	// });
}
function remove(i: number) {
	choices.value = choices.value.filter((_, _i) => _i !== i);
}
function get(): PollEditorModelValue {
	const calcAt = () => {
		return new Date(`${atDate.value} ${atTime.value}`).getTime();
	};
	const calcAfter = () => {
		let base = parseInt(after.value.toString());
		switch (unit.value) {
			// @ts-expect-error fallthrough
			case 'day': base *= 24;
			// @ts-expect-error fallthrough
			case 'hour': base *= 60;
			// @ts-expect-error fallthrough
			case 'minute': base *= 60;
			// eslint-disable-next-line no-fallthrough
			case 'second': return base *= 1000;
			default: return null;
		}
	};
	return {
		choices: choices.value,
		multiple: multiple.value,
		expiresAt: expiration.value === 'at' ? calcAt() : null,
		expiredAfter: expiration.value === 'after' ? calcAfter() : null,
	};
}
watch([choices, multiple, expiration, atDate, atTime, after, unit], () => emit('update:modelValue', get()), {
	deep: true,
});

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", { class: "zmdxowus" }, [ (choices.value.length < 2) ? (_openBlock(), _createElementBlock("p", {
          key: 0,
          class: "caution"
        }, [ _hoisted_1, _toDisplayString(_unref(i18n).ts._poll.noOnlyOneChoice) ])) : _createCommentVNode("v-if", true), _createElementVNode("ul", null, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(choices.value, (choice, i) => {
          return (_openBlock(), _createElementBlock("li", { key: i }, [
            _createVNode(MkInput, {
              class: "input",
              small: "",
              modelValue: choice,
              placeholder: _unref(i18n).tsx._poll.choiceN({ n: i + 1 }),
              "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => (onInput(i, $event)))
            }, null, 8 /* PROPS */, ["modelValue", "placeholder"]),
            _createElementVNode("button", {
              class: "_button",
              onClick: _cache[1] || (_cache[1] = ($event: any) => (remove(i)))
            }, [
              _hoisted_2
            ])
          ]))
        }), 128 /* KEYED_FRAGMENT */)) ]), (choices.value.length < 10) ? (_openBlock(), _createBlock(MkButton, {
          key: 0,
          class: "add",
          onClick: add
        }, {
          default: _withCtx(() => [
            _createTextVNode(_toDisplayString(_unref(i18n).ts.add), 1 /* TEXT */)
          ]),
          _: 1 /* STABLE */
        })) : (_openBlock(), _createBlock(MkButton, {
          key: 1,
          class: "add",
          disabled: ""
        }, {
          default: _withCtx(() => [
            _createTextVNode(_toDisplayString(_unref(i18n).ts._poll.noMore), 1 /* TEXT */)
          ]),
          _: 1 /* STABLE */
        })), _createVNode(MkSwitch, {
        modelValue: multiple.value,
        "onUpdate:modelValue": _cache[2] || (_cache[2] = ($event: any) => ((multiple).value = $event))
      }, {
        default: _withCtx(() => [
          _createTextVNode(_toDisplayString(_unref(i18n).ts._poll.canMultipleVote), 1 /* TEXT */)
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["modelValue"]), _createElementVNode("section", null, [ _createElementVNode("div", null, [ _createVNode(MkSelect, {
            items: _unref(expirationDef),
            small: "",
            modelValue: _unref(expiration),
            "onUpdate:modelValue": _cache[3] || (_cache[3] = ($event: any) => ((expiration).value = $event))
          }, {
            label: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(i18n).ts._poll.expiration), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["items", "modelValue"]), (_unref(expiration) === 'at') ? (_openBlock(), _createElementBlock("section", { key: 0 }, [ _createVNode(MkInput, {
                small: "",
                type: "date",
                class: "input",
                modelValue: atDate.value,
                "onUpdate:modelValue": _cache[4] || (_cache[4] = ($event: any) => ((atDate).value = $event))
              }, {
                label: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._poll.deadlineDate), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["modelValue"]), _createVNode(MkInput, {
                small: "",
                type: "time",
                class: "input",
                modelValue: atTime.value,
                "onUpdate:modelValue": _cache[5] || (_cache[5] = ($event: any) => ((atTime).value = $event))
              }, {
                label: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._poll.deadlineTime), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["modelValue"]) ])) : (_unref(expiration) === 'after') ? (_openBlock(), _createElementBlock("section", { key: 1 }, [ _createVNode(MkInput, {
                  small: "",
                  type: "number",
                  min: 1,
                  class: "input",
                  modelValue: after.value,
                  "onUpdate:modelValue": _cache[6] || (_cache[6] = ($event: any) => ((after).value = $event))
                }, {
                  label: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_unref(i18n).ts._poll.duration), 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["min", "modelValue"]), _createVNode(MkSelect, {
                  items: _unref(unitDef),
                  small: "",
                  modelValue: _unref(unit),
                  "onUpdate:modelValue": _cache[7] || (_cache[7] = ($event: any) => ((unit).value = $event))
                }, null, 8 /* PROPS */, ["items", "modelValue"]) ])) : _createCommentVNode("v-if", true) ]) ]) ]))
}
}

})
