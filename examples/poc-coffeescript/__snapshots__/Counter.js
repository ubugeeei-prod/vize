import { h as _h, toDisplayString as _toDisplayString, withDirectives as _withDirectives, vModelText as _vModelText, Fragment as _Fragment, renderList as _renderList } from "vue";
import { ref } from "vue";

export default {
  props: ["title"],
  setup(__props) {
    const count = ref(0);
    const done = ref(false);
    const name = ref("world");
    const items = ref(["write", "compile", "ship"]);

    // Clicking past five marks the counter done.
    const increment = function () {
      count.value += 1;
      return done.value = count.value > 5;
    };
    return () => _h("section", { class: "counter" }, [
      _h("h1", null, _toDisplayString(__props.title)),
      _h("button", { type: "button", class: done.value ? "hot" : "cold", onClick: increment }, " Clicked " + _toDisplayString(count.value) + " times "),
      count.value > 3 && !done.value ? _h("p", null, "Hot!") : _h("p", null, "Keep clicking"),
      _withDirectives(_h("input", { "onUpdate:modelValue": ($event) => (name.value = $event) }), [[_vModelText, name.value]]),
      _h("p", null, _toDisplayString(`Hello, ${name.value}!`)),
      _h("ul", null, [_h(_Fragment, null, _renderList(items.value, (item) => _h("li", null, _toDisplayString(item.toUpperCase()))))]),
    ]);
  },
};
