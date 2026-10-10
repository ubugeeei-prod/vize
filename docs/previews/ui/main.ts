import { createApp, createSSRApp, h } from "vue";
import { examples, composables } from "virtual:vize-ui-examples";
import "./layers.css";
import "@vizejs/ui/base.css";
import "@vizejs/ui/theme-preset-paper.css";
import "@vizejs/ui/theme-preset-signal.css";
import "@vizejs/ui/theme-preset-atelier.css";
import "@vizejs/ui/component-button.css";
import "@vizejs/ui/component-input.css";
import "@vizejs/ui/component-checkbox.css";
import "@vizejs/ui/component-switch.css";
import "@vizejs/ui/component-tabs.css";
import "@vizejs/ui/component-dialog.css";
import "@vizejs/ui/component-alert.css";
import "@vizejs/ui/component-card.css";
import "./preview.css";

const params = new URLSearchParams(location.search);
const family = params.get("family") ?? "button";
const composable = params.get("composable");
const name = composable ?? family;
const example = composable == null ? examples[family] : composables[composable];
const theme = params.get("theme") ?? "paper";
const themes = ["paper", "signal", "atelier"];
document.documentElement.dataset.vizeTheme = themes.includes(theme) ? theme : "paper";

if (example == null) {
  document.querySelector("#app")!.textContent = `No example exists for ${name}.`;
} else {
  const { default: Component } = await example.load();
  document.title = `${example.title} — Vize example`;
  createApp({
    render: () =>
      h("div", { class: "preview-shell" }, [
        h("header", [
          h("strong", example.title),
          composable == null
            ? h("label", [
                "Style ",
                h(
                  "select",
                  {
                    "aria-label": "Style preset",
                    value: document.documentElement.dataset.vizeTheme,
                    onChange: (event: Event) => {
                      document.documentElement.dataset.vizeTheme = (
                        event.target as HTMLSelectElement
                      ).value;
                    },
                  },
                  themes.map((preset) => h("option", { value: preset }, preset)),
                ),
              ])
            : h("span", "Reactive Vue example"),
        ]),
        h("section", { class: "preview-stage", "aria-label": `${example.title} example` }, [
          composable == null
            ? h(Component)
            : h("div", { "data-hydration-root": "", innerHTML: composables[composable]!.ssrHtml }),
        ]),
        h("footer", [
          h(
            "a",
            {
              href: `/guide/${composable == null ? "ui" : "composables"}/${name}`,
              target: "_parent",
            },
            "View code and API",
          ),
        ]),
      ]),
  }).mount("#app");
  if (composable != null) {
    const stage = document.querySelector("[data-hydration-root]")!;
    const originalElements = [...stage.querySelectorAll("*")];
    const originalMarkup = stage.innerHTML;
    createSSRApp(Component).mount(stage);
    document.documentElement.dataset.previewHydrationRetained = String(
      originalMarkup === stage.innerHTML &&
        originalElements.every((element, index) => element === stage.querySelectorAll("*")[index]),
    );
  }
  document.documentElement.dataset.previewReady = name;
  document.documentElement.dataset.previewSourceSha256 = example.sourceSha256;
}
