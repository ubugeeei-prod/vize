import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"

import { reactive, computed, watch } from 'vue'
import * as Misskey from 'misskey-js'
import MkSelect from '@/components/MkSelect.vue'
import MkInput from '@/components/MkInput.vue'
import MkSwitch from '@/components/MkSwitch.vue'
import MkRadios from '@/components/MkRadios.vue'
import MkButton from '@/components/MkButton.vue'
import MkRange from '@/components/MkRange.vue'
import { i18n } from '@/i18n.js'
import { instance } from '@/instance.js'
import { deepClone } from '@/utility/clone.js'
import { prefer } from '@/preferences.js'
import type { MkSelectItem } from '@/components/MkSelect.vue'
import type { StatusbarStore } from '@/preferences/def.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'statusbar.statusbar',
  props: {
    _id: { type: String, required: true },
    userLists: { type: Array, required: true }
  },
  setup(__props: any) {

const props = __props
const statusbar = reactive<StatusbarStore>(deepClone(prefer.s.statusbars.find(x => x.id === props._id)!));
const statusbarTypeDef = computed(() => {
	const items = [
		{ label: 'RSS', value: 'rss' },
	] satisfies MkSelectItem[];
	if (instance.federation !== 'none') {
		items.push({ label: 'Federation', value: 'federation' });
	}
	if (props.userLists != null) {
		items.push({ label: i18n.ts.userList, value: 'userList' });
	}
	return items;
});
const userListsDef = computed(() => {
	return (props.userLists ?? []).map(x => ({ label: x.name, value: x.id })) satisfies MkSelectItem[];
});
watch(() => statusbar.type, () => {
	if (statusbar.type === 'rss') {
		statusbar.name = 'NEWS';
		statusbar.props.url = 'http://feeds.afpbb.com/rss/afpbb/afpbbnews';
		statusbar.props.shuffle = true;
		statusbar.props.refreshIntervalSec = 120;
		statusbar.props.display = 'marquee';
		statusbar.props.marqueeDuration = 100;
		statusbar.props.marqueeReverse = false;
	} else if (statusbar.type === 'federation') {
		statusbar.name = 'FEDERATION';
		statusbar.props.refreshIntervalSec = 120;
		statusbar.props.display = 'marquee';
		statusbar.props.marqueeDuration = 100;
		statusbar.props.marqueeReverse = false;
		statusbar.props.colored = false;
	} else if (statusbar.type === 'userList') {
		statusbar.name = 'LIST TL';
		statusbar.props.refreshIntervalSec = 120;
		statusbar.props.display = 'marquee';
		statusbar.props.marqueeDuration = 100;
		statusbar.props.marqueeReverse = false;
	}
});
watch(statusbar, save);
async function save() {
	const i = prefer.s.statusbars.findIndex(x => x.id === props._id);
	const statusbars = deepClone(prefer.s.statusbars);
	statusbars[i] = deepClone(statusbar);
	prefer.commit('statusbars', statusbars);
}
function del() {
	prefer.commit('statusbars', prefer.s.statusbars.filter(x => x.id !== props._id));
}

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", { class: "_gaps_m" }, [ _createVNode(MkSelect, {
        items: statusbarTypeDef.value,
        modelValue: statusbar.type,
        "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((statusbar.type) = $event))
      }, {
        label: _withCtx(() => [
          _createTextVNode(_toDisplayString(_unref(i18n).ts.type), 1 /* TEXT */)
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["items", "modelValue"]), _createVNode(MkInput, {
        manualSave: "",
        modelValue: statusbar.name,
        "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event: any) => ((statusbar.name) = $event))
      }, {
        label: _withCtx(() => [
          _createTextVNode(_toDisplayString(_unref(i18n).ts.label), 1 /* TEXT */)
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["modelValue"]), _createVNode(MkSwitch, {
        modelValue: statusbar.black,
        "onUpdate:modelValue": _cache[2] || (_cache[2] = ($event: any) => ((statusbar.black) = $event))
      }, {
        label: _withCtx(() => [
          _createTextVNode("Black")
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["modelValue"]), _createVNode(MkRadios, {
        options: [
  			{ value: 'verySmall', label: _unref(i18n).ts.small + '+' },
  			{ value: 'small', label: _unref(i18n).ts.small },
  			{ value: 'medium', label: _unref(i18n).ts.medium },
  			{ value: 'large', label: _unref(i18n).ts.large },
  			{ value: 'veryLarge', label: _unref(i18n).ts.large + '+' },
  		],
        modelValue: statusbar.size,
        "onUpdate:modelValue": _cache[3] || (_cache[3] = ($event: any) => ((statusbar.size) = $event))
      }, {
        label: _withCtx(() => [
          _createTextVNode(_toDisplayString(_unref(i18n).ts.size), 1 /* TEXT */)
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["options", "modelValue"]), (statusbar.type === 'rss') ? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [ _createVNode(MkInput, {
            manualSave: "",
            type: "url",
            modelValue: statusbar.props.url,
            "onUpdate:modelValue": _cache[4] || (_cache[4] = ($event: any) => ((statusbar.props.url) = $event))
          }, {
            label: _withCtx(() => [
              _createTextVNode("URL")
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["modelValue"]), _createVNode(MkSwitch, {
            modelValue: statusbar.props.shuffle,
            "onUpdate:modelValue": _cache[5] || (_cache[5] = ($event: any) => ((statusbar.props.shuffle) = $event))
          }, {
            label: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(i18n).ts.shuffle), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["modelValue"]), _createVNode(MkInput, {
            manualSave: "",
            type: "number",
            min: 1,
            modelValue: statusbar.props.refreshIntervalSec,
            "onUpdate:modelValue": _cache[6] || (_cache[6] = ($event: any) => ((statusbar.props.refreshIntervalSec) = $event))
          }, {
            label: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(i18n).ts.refreshInterval), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["min", "modelValue"]), _createVNode(MkRange, {
            min: 5,
            max: 150,
            step: 1,
            modelValue: statusbar.props.marqueeDuration,
            "onUpdate:modelValue": _cache[7] || (_cache[7] = ($event: any) => ((statusbar.props.marqueeDuration) = $event))
          }, {
            label: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(i18n).ts.speed), 1 /* TEXT */)
            ]),
            caption: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(i18n).ts.fast) + " &lt;-&gt; " + _toDisplayString(_unref(i18n).ts.slow), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["min", "max", "step", "modelValue"]), _createVNode(MkSwitch, {
            modelValue: statusbar.props.marqueeReverse,
            "onUpdate:modelValue": _cache[8] || (_cache[8] = ($event: any) => ((statusbar.props.marqueeReverse) = $event))
          }, {
            label: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(i18n).ts.reverse), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["modelValue"]) ], 64 /* STABLE_FRAGMENT */)) : (statusbar.type === 'federation') ? (_openBlock(), _createElementBlock(_Fragment, { key: 1 }, [ _createVNode(MkInput, {
              manualSave: "",
              type: "number",
              min: 1,
              modelValue: statusbar.props.refreshIntervalSec,
              "onUpdate:modelValue": _cache[9] || (_cache[9] = ($event: any) => ((statusbar.props.refreshIntervalSec) = $event))
            }, {
              label: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts.refreshInterval), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["min", "modelValue"]), _createVNode(MkRange, {
              min: 5,
              max: 150,
              step: 1,
              modelValue: statusbar.props.marqueeDuration,
              "onUpdate:modelValue": _cache[10] || (_cache[10] = ($event: any) => ((statusbar.props.marqueeDuration) = $event))
            }, {
              label: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts.speed), 1 /* TEXT */)
              ]),
              caption: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts.fast) + " &lt;-&gt; " + _toDisplayString(_unref(i18n).ts.slow), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["min", "max", "step", "modelValue"]), _createVNode(MkSwitch, {
              modelValue: statusbar.props.marqueeReverse,
              "onUpdate:modelValue": _cache[11] || (_cache[11] = ($event: any) => ((statusbar.props.marqueeReverse) = $event))
            }, {
              label: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts.reverse), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["modelValue"]), _createVNode(MkSwitch, {
              modelValue: statusbar.props.colored,
              "onUpdate:modelValue": _cache[12] || (_cache[12] = ($event: any) => ((statusbar.props.colored) = $event))
            }, {
              label: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts.colored), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["modelValue"]) ], 64 /* STABLE_FRAGMENT */)) : (statusbar.type === 'userList' && __props.userLists != null) ? (_openBlock(), _createElementBlock(_Fragment, { key: 2 }, [ _createVNode(MkSelect, {
              items: userListsDef.value,
              modelValue: statusbar.props.userListId,
              "onUpdate:modelValue": _cache[13] || (_cache[13] = ($event: any) => ((statusbar.props.userListId) = $event))
            }, {
              label: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts.userList), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["items", "modelValue"]), _createVNode(MkInput, {
              manualSave: "",
              type: "number",
              modelValue: statusbar.props.refreshIntervalSec,
              "onUpdate:modelValue": _cache[14] || (_cache[14] = ($event: any) => ((statusbar.props.refreshIntervalSec) = $event))
            }, {
              label: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts.refreshInterval), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["modelValue"]), _createVNode(MkRange, {
              min: 5,
              max: 150,
              step: 1,
              modelValue: statusbar.props.marqueeDuration,
              "onUpdate:modelValue": _cache[15] || (_cache[15] = ($event: any) => ((statusbar.props.marqueeDuration) = $event))
            }, {
              label: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts.speed), 1 /* TEXT */)
              ]),
              caption: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts.fast) + " &lt;-&gt; " + _toDisplayString(_unref(i18n).ts.slow), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["min", "max", "step", "modelValue"]), _createVNode(MkSwitch, {
              modelValue: statusbar.props.marqueeReverse,
              "onUpdate:modelValue": _cache[16] || (_cache[16] = ($event: any) => ((statusbar.props.marqueeReverse) = $event))
            }, {
              label: _withCtx(() => [
                _createTextVNode(_toDisplayString(_unref(i18n).ts.reverse), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["modelValue"]) ], 64 /* STABLE_FRAGMENT */)) : _createCommentVNode("v-if", true), _createElementVNode("div", { class: "_buttons" }, [ _createVNode(MkButton, {
          danger: "",
          onClick: del
        }, {
          default: _withCtx(() => [
            _createTextVNode(_toDisplayString(_unref(i18n).ts.remove), 1 /* TEXT */)
          ]),
          _: 1 /* STABLE */
        }) ]) ]))
}
}

})
