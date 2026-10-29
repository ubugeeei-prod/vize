export const componentClassExpected = `import { useCssModule as _useCssModule } from 'vue'
import { normalizeClass as _normalizeClass, openBlock as _openBlock, createBlock as _createBlock } from "vue"


function _sfc_render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createBlock($setup["Child"], {
    channel: $setup.channel,
    full: true,
    class: _normalizeClass($setup.$style.subscribe)
  }, null, 8 /* PROPS */, ["channel", "class"]))
}
import { ref } from 'vue'
import Child from './Child.vue'


export default {
  __name: 'ComponentClass',
  __cssModules: {
    "$style": {
      "subscribe": "JA83LG_subscribe"
    }
  },
  render: _sfc_render,
  setup(__props, { expose: __expose }) {
  __expose();

const $style = _useCssModule()

const channel = ref({});

const __returned__ = { Child, ref, $style, channel }
Object.defineProperty(__returned__, '__isScriptSetup', { enumerable: false, value: true })
return __returned__
}

}`;

export const antDesignDomExpected = `import { createVNode as _createVNode, openBlock as _openBlock, createBlock as _createBlock, withCtx as _withCtx } from "vue";
function _sfc_render(_ctx, _cache, $props, $setup, $data, $options) {
  return _openBlock(), _createBlock($setup["Form"].Item, { label: "Teacher" }, {
    default: _withCtx(() => [_createVNode($setup["Input"])]),
    _: 1
  });
}
import { Form, Input } from "ant-design-vue";
export default {
  __name: "AntDesignTeacher",
  render: _sfc_render,
  setup(__props, { expose: __expose }) {
    __expose();
    const __returned__ = {
      Form,
      Input
    };
    Object.defineProperty(__returned__, "__isScriptSetup", {
      enumerable: false,
      value: true
    });
    return __returned__;
  }
};`;
