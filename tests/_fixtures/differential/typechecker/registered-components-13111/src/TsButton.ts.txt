import { defineComponent, h } from "vue";

export default defineComponent({
  props: { size: { type: String, required: true } },
  setup(props) {
    return () => h("button", props.size);
  },
});
