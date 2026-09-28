import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vite-plus/test";
import { createEmptyCodeOutputs } from "../atelier/codeOutputs";
import OutputView from "./OutputView.vue";

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
          CodeHighlight: { props: ["code"], template: "<pre>{{ code }}</pre>" },
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
