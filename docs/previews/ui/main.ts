import { createApp, h } from "vue";
import { examples } from "virtual:vize-ui-examples";
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
const example = examples[family];
const theme = params.get("theme") ?? "paper";
const themes = ["paper", "signal", "atelier"];
document.documentElement.dataset.vizeTheme = themes.includes(theme) ? theme : "paper";

if (example == null) {
  document.querySelector("#app")!.textContent = `No example exists for ${family}.`;
} else {
  const { default: Component } = await example.load();
  document.title = `${example.title} — Vize example`;
  createApp({
    render: () =>
      h("div", { class: "preview-shell" }, [
        h("header", [
          h("strong", example.title),
          h("label", [
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
          ]),
        ]),
        h("section", { class: "preview-stage", "aria-label": `${example.title} example` }, [
          h(Component),
        ]),
        h("footer", [
          h("a", { href: `/guide/ui/${family}`, target: "_parent" }, "View code and API"),
        ]),
      ]),
  }).mount("#app");
  document.documentElement.dataset.previewReady = family;
  document.documentElement.dataset.previewSourceSha256 = example.sourceSha256;
}
