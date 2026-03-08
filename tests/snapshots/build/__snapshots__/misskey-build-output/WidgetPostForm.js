import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock } from "vue"

import { useWidgetPropsManager } from './widget.js'
import type { WidgetComponentEmits, WidgetComponentExpose, WidgetComponentProps } from './widget.js'
import type { FormWithDefault, GetFormResultType } from '@/utility/form.js'
import MkPostForm from '@/components/MkPostForm.vue'

type WidgetProps = GetFormResultType<typeof widgetPropsDef>;
const name = 'postForm';

export default /*@__PURE__*/_defineComponent({
  __name: 'WidgetPostForm',
  setup(__props: any, { expose: __expose, emit: __emit }) {

const emit = __emit
const props = __props
const widgetPropsDef = {
} satisfies FormWithDefault;
const { widgetProps, configure } = useWidgetPropsManager(name,
	widgetPropsDef,
	props,
	emit,
);
__expose({
	name,
	configure,
	id: props.widget ? props.widget.id : null,
})

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createBlock(MkPostForm, {
      "data-cy-mkw-postForm": "",
      class: "_panel mkw-post-form",
      fixed: true,
      autofocus: false
    }, null, 8 /* PROPS */, ["fixed", "autofocus"]))
}
}

})
