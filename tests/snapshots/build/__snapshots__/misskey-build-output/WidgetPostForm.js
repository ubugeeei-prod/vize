import { openBlock as _openBlock, createBlock as _createBlock } from "vue";
import { useWidgetPropsManager } from "./widget.js";
import MkPostForm from "@/components/MkPostForm.vue";
const name = "postForm";
export default {
  __name: "WidgetPostForm",
  setup(__props, { expose: __expose, emit: __emit }) {
    const emit = __emit;
    const props = __props;
    const widgetPropsDef = {};
    const { widgetProps, configure } = useWidgetPropsManager(name, widgetPropsDef, props, emit);
    __expose({
      name,
      configure,
      id: props.widget ? props.widget.id : null
    });
    return (_ctx, _cache) => {
      return _openBlock(), _createBlock(MkPostForm, {
        "data-cy-mkw-postForm": "",
        class: "_panel mkw-post-form",
        fixed: true,
        autofocus: false
      }, null, 8, ["fixed", "autofocus"]);
    };
  }
};
