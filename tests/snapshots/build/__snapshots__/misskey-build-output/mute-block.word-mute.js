import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("br")
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-device-floppy" })
import { ref, watch } from 'vue'
import MkTextarea from '@/components/MkTextarea.vue'
import MkButton from '@/components/MkButton.vue'
import * as os from '@/os.js'
import { i18n } from '@/i18n.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'mute-block.word-mute',
  props: {
    muted: { type: Array, required: true }
  },
  emits: ["save"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
const render = (mutedWords: (string | string[])[]) => mutedWords.map(x => {
	if (Array.isArray(x)) {
		return x.join(' ');
	} else {
		return x;
	}
}).join('\n');
const mutedWords = ref(render(props.muted));
const changed = ref(false);
watch(mutedWords, () => {
	changed.value = true;
});
async function save() {
	const parseMutes = (mutes: string) => {
		// split into lines, remove empty lines and unnecessary whitespace
		let lines = mutes.trim().split('\n').map(line => line.trim()).filter(line => line !== '') as (string | string[])[];

		// check each line if it is a RegExp or not
		for (let i = 0; i < lines.length; i++) {
			const line = lines[i] as string;
			const regexp = line.match(/^\/(.+)\/(.*)$/);
			if (regexp) {
				// check that the RegExp is valid
				try {
					new RegExp(regexp[1], regexp[2]);
					// note that regex lines will not be split by spaces!
				} catch (err: any) {
					// invalid syntax: do not save, do not reset changed flag
					os.alert({
						type: 'error',
						title: i18n.ts.regexpError,
						text: i18n.tsx.regexpErrorDescription({ tab: 'word mute', line: i + 1 }) + '\n' + err.toString(),
					});
					// re-throw error so these invalid settings are not saved
					throw err;
				}
			} else {
				lines[i] = line.split(' ');
			}
		}

		return lines;
	};
	let parsed;
	try {
		parsed = parseMutes(mutedWords.value);
	} catch (err) {
		// already displayed error message in parseMutes
		return;
	}
	emit('save', parsed);
	changed.value = false;
}

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", { class: "_gaps_m" }, [ _createElementVNode("div", null, [ _createVNode(MkTextarea, {
          modelValue: mutedWords.value,
          "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((mutedWords).value = $event))
        }, {
          caption: _withCtx(() => [
            _createTextVNode(_toDisplayString(_unref(i18n).ts._wordMute.muteWordsDescription), 1 /* TEXT */),
            _hoisted_1,
            _createTextVNode(_toDisplayString(_unref(i18n).ts._wordMute.muteWordsDescription2), 1 /* TEXT */)
          ]),
          default: _withCtx(() => [
            _createElementVNode("span", null, _toDisplayString(_unref(i18n).ts._wordMute.muteWords), 1 /* TEXT */)
          ]),
          _: 1 /* STABLE */
        }, 8 /* PROPS */, ["modelValue"]) ]), _createVNode(MkButton, {
        primary: "",
        inline: "",
        disabled: !changed.value,
        onClick: _cache[1] || (_cache[1] = ($event: any) => (save()))
      }, {
        default: _withCtx(() => [
          _hoisted_2,
          _createTextVNode(" "),
          _createTextVNode(_toDisplayString(_unref(i18n).ts.save), 1 /* TEXT */)
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["disabled"]) ]))
}
}

})
