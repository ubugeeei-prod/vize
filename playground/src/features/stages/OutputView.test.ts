import { mount } from "@vue/test-utils";
import { defineComponent, h } from "vue";
import { describe, expect, it } from "vite-plus/test";
import { createEmptyCodeOutputs } from "../atelier/codeOutputs";
import OutputView from "./OutputView.vue";

const CodeHighlightStub = defineComponent({
  props: { code: { type: String, required: true } },
  setup(props) {
    return () => h("pre", props.code);
  },
});

describe("product output boundaries", () => {
  it("shows the captured L4 backend page separately from assembled SFC code", async () => {
    const outputs = createEmptyCodeOutputs();
    outputs.dom.code = "assembled SFC module";
    const wrapper = mount(OutputView, {
      props: {
        outputs,
        target: "dom",
        theme: "light",
        backendPages: [{ step: "emit", text: "captured L4 render" }],
      },
      global: {
        stubs: {
          CodeHighlight: CodeHighlightStub,
        },
      },
    });
    expect(wrapper.find("pre").text()).toBe("captured L4 render");
    await wrapper
      .findAll("button")
      .find((button) => button.text() === "Assembled SFC module")
      ?.trigger("click");
    expect(wrapper.find("pre").text()).toBe("assembled SFC module");
    wrapper.unmount();
  });
});
