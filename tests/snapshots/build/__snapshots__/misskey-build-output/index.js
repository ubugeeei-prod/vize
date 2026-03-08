import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-server" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-selector" })
import { onUnmounted, ref } from 'vue'
import * as Misskey from 'misskey-js'
import { useWidgetPropsManager } from '../widget.js'
import type { WidgetComponentProps, WidgetComponentEmits, WidgetComponentExpose } from '../widget.js'
import XCpuMemory from './cpu-mem.vue'
import XNet from './net.vue'
import XCpu from './cpu.vue'
import XMemory from './mem.vue'
import XDisk from './disk.vue'
import MkContainer from '@/components/MkContainer.vue'
import type { FormWithDefault, GetFormResultType } from '@/utility/form.js'
import { misskeyApiGet } from '@/utility/misskey-api.js'
import { useStream } from '@/stream.js'
import { i18n } from '@/i18n.js'

type WidgetProps = GetFormResultType<typeof widgetPropsDef>;
const name = 'serverMetric';

export default /*@__PURE__*/_defineComponent({
  __name: 'index',
  setup(__props: any, { expose: __expose, emit: __emit }) {

const emit = __emit
const props = __props
const widgetPropsDef = {
	showHeader: {
		type: 'boolean',
		label: i18n.ts._widgetOptions.showHeader,
		default: true,
	},
	transparent: {
		type: 'boolean',
		label: i18n.ts._widgetOptions.transparent,
		default: false,
	},
	view: {
		type: 'number',
		default: 0,
		hidden: true,
	},
} satisfies FormWithDefault;
const { widgetProps, configure, save } = useWidgetPropsManager(name,
	widgetPropsDef,
	props,
	emit,
);
const meta = ref<Misskey.entities.ServerInfoResponse | null>(null);
misskeyApiGet('server-info', {}).then(res => {
	meta.value = res;
});
const toggleView = () => {
	if (widgetProps.view === 4) {
		widgetProps.view = 0;
	} else {
		widgetProps.view++;
	}
	save();
};
const connection = useStream().useChannel('serverStats');
onUnmounted(() => {
	connection.dispose();
});
__expose({
	name,
	configure,
	id: props.widget ? props.widget.id : null,
})

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createBlock(MkContainer, {
      showHeader: _unref(widgetProps).showHeader,
      naked: _unref(widgetProps).transparent
    }, {
      icon: _withCtx(() => [
        _hoisted_1
      ]),
      header: _withCtx(() => [
        _createTextVNode(_toDisplayString(_unref(i18n).ts._widgets.serverMetric), 1 /* TEXT */)
      ]),
      func: _withCtx(({ buttonStyleClass }) => [
        _createElementVNode("button", {
          class: _normalizeClass(["_button", buttonStyleClass]),
          onClick: _cache[0] || (_cache[0] = ($event: any) => (toggleView()))
        }, [
          _hoisted_2
        ])
      ]),
      default: _withCtx(() => [
        (meta.value)
          ? (_openBlock(), _createElementBlock("div", {
            key: 0,
            "data-cy-mkw-serverMetric": "",
            class: "mkw-serverMetric"
          }, [
            (_unref(widgetProps).view === 0)
              ? (_openBlock(), _createBlock(XCpuMemory, {
                key: 0,
                connection: _unref(connection),
                meta: meta.value
              }, null, 8 /* PROPS */, ["connection", "meta"]))
              : (_unref(widgetProps).view === 1)
                ? (_openBlock(), _createBlock(XNet, {
                  key: 1,
                  connection: _unref(connection),
                  meta: meta.value
                }, null, 8 /* PROPS */, ["connection", "meta"]))
              : (_unref(widgetProps).view === 2)
                ? (_openBlock(), _createBlock(XCpu, {
                  key: 2,
                  connection: _unref(connection),
                  meta: meta.value
                }, null, 8 /* PROPS */, ["connection", "meta"]))
              : (_unref(widgetProps).view === 3)
                ? (_openBlock(), _createBlock(XMemory, {
                  key: 3,
                  connection: _unref(connection),
                  meta: meta.value
                }, null, 8 /* PROPS */, ["connection", "meta"]))
              : (_unref(widgetProps).view === 4)
                ? (_openBlock(), _createBlock(XDisk, {
                  key: 4,
                  meta: meta.value
                }, null, 8 /* PROPS */, ["meta"]))
              : _createCommentVNode("v-if", true)
          ]))
          : _createCommentVNode("v-if", true)
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["showHeader", "naked"]))
}
}

})
