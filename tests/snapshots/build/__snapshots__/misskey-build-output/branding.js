import { withAsyncContext as _withAsyncContext } from "vue";
import { openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue";
const _hoisted_1 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-link" });
const _hoisted_2 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-link" });
const _hoisted_3 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-link" });
const _hoisted_4 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-link" });
const _hoisted_5 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-link" });
const _hoisted_6 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-link" });
const _hoisted_7 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-link" });
const _hoisted_8 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-link" });
const _hoisted_9 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-link" });
const _hoisted_10 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-link" });
const _hoisted_11 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-check" });
import { ref, computed } from "vue";
import JSON5 from "json5";
import { host } from "@@/js/config.js";
import MkInput from "@/components/MkInput.vue";
import MkTextarea from "@/components/MkTextarea.vue";
import * as os from "@/os.js";
import { misskeyApi } from "@/utility/misskey-api.js";
import { instance, fetchInstance } from "@/instance.js";
import { i18n } from "@/i18n.js";
import { definePage } from "@/page.js";
import MkButton from "@/components/MkButton.vue";
import MkColorInput from "@/components/MkColorInput.vue";
import MkRadios from "@/components/MkRadios.vue";
import MkSwitch from "@/components/MkSwitch.vue";
export default {
  __name: "branding",
  async setup(__props) {
    let __temp, __restore;
    const meta = ([__temp, __restore] = _withAsyncContext(() => misskeyApi("admin/meta")), __temp = await __temp, __restore(), __temp);
    // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
    const entrancePageStyle = ref(meta.clientOptions.entrancePageStyle ?? "classic");
    // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
    const showTimelineForVisitor = ref(meta.clientOptions.showTimelineForVisitor ?? true);
    // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
    const showActivitiesForVisitor = ref(meta.clientOptions.showActivitiesForVisitor ?? true);
    const iconUrl = ref(meta.iconUrl);
    const app192IconUrl = ref(meta.app192IconUrl);
    const app512IconUrl = ref(meta.app512IconUrl);
    const bannerUrl = ref(meta.bannerUrl);
    const backgroundImageUrl = ref(meta.backgroundImageUrl);
    const themeColor = ref(meta.themeColor);
    const defaultLightTheme = ref(meta.defaultLightTheme);
    const defaultDarkTheme = ref(meta.defaultDarkTheme);
    const serverErrorImageUrl = ref(meta.serverErrorImageUrl);
    const infoImageUrl = ref(meta.infoImageUrl);
    const notFoundImageUrl = ref(meta.notFoundImageUrl);
    const repositoryUrl = ref(meta.repositoryUrl);
    const feedbackUrl = ref(meta.feedbackUrl);
    const manifestJsonOverride = ref(meta.manifestJsonOverride === "" ? "{}" : JSON.stringify(JSON.parse(meta.manifestJsonOverride), null, "  "));
    function save() {
      os.apiWithDialog("admin/update-meta", {
        clientOptions: {
          entrancePageStyle: entrancePageStyle.value,
          showTimelineForVisitor: showTimelineForVisitor.value,
          showActivitiesForVisitor: showActivitiesForVisitor.value
        },
        iconUrl: iconUrl.value,
        app192IconUrl: app192IconUrl.value,
        app512IconUrl: app512IconUrl.value,
        bannerUrl: bannerUrl.value,
        backgroundImageUrl: backgroundImageUrl.value,
        themeColor: themeColor.value === "" ? null : themeColor.value,
        defaultLightTheme: defaultLightTheme.value === "" ? null : defaultLightTheme.value,
        defaultDarkTheme: defaultDarkTheme.value === "" ? null : defaultDarkTheme.value,
        infoImageUrl: infoImageUrl.value === "" ? null : infoImageUrl.value,
        notFoundImageUrl: notFoundImageUrl.value === "" ? null : notFoundImageUrl.value,
        serverErrorImageUrl: serverErrorImageUrl.value === "" ? null : serverErrorImageUrl.value,
        repositoryUrl: repositoryUrl.value === "" ? null : repositoryUrl.value,
        feedbackUrl: feedbackUrl.value === "" ? null : feedbackUrl.value,
        manifestJsonOverride: manifestJsonOverride.value === "" ? "{}" : JSON.stringify(JSON5.parse(manifestJsonOverride.value))
      }).then(() => {
        fetchInstance(true);
      });
    }
    const headerTabs = computed(() => []);
    definePage(() => ({
      title: i18n.ts.branding,
      icon: "ti ti-paint"
    }));
    return (_ctx, _cache) => {
      const _component_SearchLabel = _resolveComponent("SearchLabel");
      const _component_SearchMarker = _resolveComponent("SearchMarker");
      const _component_PageWithHeader = _resolveComponent("PageWithHeader");
      return _openBlock(), _createBlock(_component_PageWithHeader, { tabs: headerTabs.value }, {
        footer: _withCtx(() => [_createElementVNode(
          "div",
          { class: _normalizeClass(_ctx.$style.footer) },
          [_createElementVNode("div", {
            class: "_spacer",
            style: "--MI_SPACER-w: 700px; --MI_SPACER-min: 16px; --MI_SPACER-max: 16px;"
          }, [_createVNode(MkButton, {
            primary: "",
            rounded: "",
            onClick: save
          }, {
            default: _withCtx(() => [
              _hoisted_11,
              _createTextVNode(" "),
              _createTextVNode(
                _toDisplayString(_unref(i18n).ts.save),
                1
                /* TEXT */
              )
            ]),
            _: 1
          })])],
          2
          /* CLASS */
        )]),
        default: _withCtx(() => [_createElementVNode("div", {
          class: "_spacer",
          style: "--MI_SPACER-w: 700px; --MI_SPACER-min: 16px; --MI_SPACER-max: 32px;"
        }, [_createVNode(_component_SearchMarker, {
          path: "/admin/branding",
          label: _unref(i18n).ts.branding,
          keywords: ["branding"],
          icon: "ti ti-paint"
        }, {
          default: _withCtx(() => [_createElementVNode("div", { class: "_gaps_m" }, [
            _createVNode(_component_SearchMarker, { keywords: [
              "entrance",
              "welcome",
              "landing",
              "front",
              "home",
              "page",
              "style"
            ] }, {
              default: _withCtx(() => [_createVNode(MkRadios, {
                options: [{ value: "classic" }, { value: "simple" }],
                modelValue: entrancePageStyle.value,
                "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event) => entrancePageStyle.value = $event)
              }, {
                label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                  default: _withCtx(() => [_createTextVNode(
                    _toDisplayString(_unref(i18n).ts._serverSettings.entrancePageStyle),
                    1
                    /* TEXT */
                  )]),
                  _: 1
                })]),
                _: 1
              }, 8, ["options", "modelValue"])]),
              _: 1
            }, 8, ["keywords"]),
            _createVNode(_component_SearchMarker, { keywords: ["timeline"] }, {
              default: _withCtx(() => [_createVNode(MkSwitch, {
                modelValue: showTimelineForVisitor.value,
                "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event) => showTimelineForVisitor.value = $event)
              }, {
                label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                  default: _withCtx(() => [_createTextVNode(
                    _toDisplayString(_unref(i18n).ts._serverSettings.showTimelineForVisitor),
                    1
                    /* TEXT */
                  )]),
                  _: 1
                })]),
                _: 1
              }, 8, ["modelValue"])]),
              _: 1
            }, 8, ["keywords"]),
            _createVNode(_component_SearchMarker, { keywords: ["activity", "activities"] }, {
              default: _withCtx(() => [_createVNode(MkSwitch, {
                modelValue: showActivitiesForVisitor.value,
                "onUpdate:modelValue": _cache[2] || (_cache[2] = ($event) => showActivitiesForVisitor.value = $event)
              }, {
                label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                  default: _withCtx(() => [_createTextVNode(
                    _toDisplayString(_unref(i18n).ts._serverSettings.showActivitiesForVisitor),
                    1
                    /* TEXT */
                  )]),
                  _: 1
                })]),
                _: 1
              }, 8, ["modelValue"])]),
              _: 1
            }, 8, ["keywords"]),
            _createVNode(_component_SearchMarker, { keywords: ["icon", "image"] }, {
              default: _withCtx(() => [_createVNode(MkInput, {
                type: "url",
                modelValue: iconUrl.value,
                "onUpdate:modelValue": _cache[3] || (_cache[3] = ($event) => iconUrl.value = $event)
              }, {
                prefix: _withCtx(() => [_hoisted_1]),
                label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                  default: _withCtx(() => [_createTextVNode(
                    _toDisplayString(_unref(i18n).ts._serverSettings.iconUrl),
                    1
                    /* TEXT */
                  )]),
                  _: 1
                })]),
                _: 1
              }, 8, ["modelValue"])]),
              _: 1
            }, 8, ["keywords"]),
            _createVNode(_component_SearchMarker, { keywords: ["icon", "image"] }, {
              default: _withCtx(() => [_createVNode(MkInput, {
                type: "url",
                modelValue: app192IconUrl.value,
                "onUpdate:modelValue": _cache[4] || (_cache[4] = ($event) => app192IconUrl.value = $event)
              }, {
                prefix: _withCtx(() => [_hoisted_2]),
                label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                  default: _withCtx(() => [_createTextVNode(
                    _toDisplayString(_unref(i18n).ts._serverSettings.iconUrl),
                    1
                    /* TEXT */
                  ), _createTextVNode(" (App/192px)")]),
                  _: 1
                })]),
                caption: _withCtx(() => [
                  _createElementVNode(
                    "div",
                    null,
                    _toDisplayString(_unref(i18n).tsx._serverSettings.appIconDescription({ host: _unref(instance).name ?? _unref(host) })),
                    1
                    /* TEXT */
                  ),
                  _createElementVNode(
                    "div",
                    null,
                    "(" + _toDisplayString(_unref(i18n).ts._serverSettings.appIconUsageExample) + ")",
                    1
                    /* TEXT */
                  ),
                  _createElementVNode(
                    "div",
                    null,
                    _toDisplayString(_unref(i18n).ts._serverSettings.appIconStyleRecommendation),
                    1
                    /* TEXT */
                  ),
                  _createElementVNode("div", null, [_createElementVNode(
                    "strong",
                    null,
                    _toDisplayString(_unref(i18n).tsx._serverSettings.appIconResolutionMustBe({ resolution: "192x192px" })),
                    1
                    /* TEXT */
                  )])
                ]),
                _: 1
              }, 8, ["modelValue"])]),
              _: 1
            }, 8, ["keywords"]),
            _createVNode(_component_SearchMarker, { keywords: ["icon", "image"] }, {
              default: _withCtx(() => [_createVNode(MkInput, {
                type: "url",
                modelValue: app512IconUrl.value,
                "onUpdate:modelValue": _cache[5] || (_cache[5] = ($event) => app512IconUrl.value = $event)
              }, {
                prefix: _withCtx(() => [_hoisted_3]),
                label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                  default: _withCtx(() => [_createTextVNode(
                    _toDisplayString(_unref(i18n).ts._serverSettings.iconUrl),
                    1
                    /* TEXT */
                  ), _createTextVNode(" (App/512px)")]),
                  _: 1
                })]),
                caption: _withCtx(() => [
                  _createElementVNode(
                    "div",
                    null,
                    _toDisplayString(_unref(i18n).tsx._serverSettings.appIconDescription({ host: _unref(instance).name ?? _unref(host) })),
                    1
                    /* TEXT */
                  ),
                  _createElementVNode(
                    "div",
                    null,
                    "(" + _toDisplayString(_unref(i18n).ts._serverSettings.appIconUsageExample) + ")",
                    1
                    /* TEXT */
                  ),
                  _createElementVNode(
                    "div",
                    null,
                    _toDisplayString(_unref(i18n).ts._serverSettings.appIconStyleRecommendation),
                    1
                    /* TEXT */
                  ),
                  _createElementVNode("div", null, [_createElementVNode(
                    "strong",
                    null,
                    _toDisplayString(_unref(i18n).tsx._serverSettings.appIconResolutionMustBe({ resolution: "512x512px" })),
                    1
                    /* TEXT */
                  )])
                ]),
                _: 1
              }, 8, ["modelValue"])]),
              _: 1
            }, 8, ["keywords"]),
            _createVNode(_component_SearchMarker, { keywords: ["banner", "image"] }, {
              default: _withCtx(() => [_createVNode(MkInput, {
                type: "url",
                modelValue: bannerUrl.value,
                "onUpdate:modelValue": _cache[6] || (_cache[6] = ($event) => bannerUrl.value = $event)
              }, {
                prefix: _withCtx(() => [_hoisted_4]),
                label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                  default: _withCtx(() => [_createTextVNode(
                    _toDisplayString(_unref(i18n).ts.bannerUrl),
                    1
                    /* TEXT */
                  )]),
                  _: 1
                })]),
                _: 1
              }, 8, ["modelValue"])]),
              _: 1
            }, 8, ["keywords"]),
            _createVNode(_component_SearchMarker, { keywords: ["background", "image"] }, {
              default: _withCtx(() => [_createVNode(MkInput, {
                type: "url",
                modelValue: backgroundImageUrl.value,
                "onUpdate:modelValue": _cache[7] || (_cache[7] = ($event) => backgroundImageUrl.value = $event)
              }, {
                prefix: _withCtx(() => [_hoisted_5]),
                label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                  default: _withCtx(() => [_createTextVNode(
                    _toDisplayString(_unref(i18n).ts.backgroundImageUrl),
                    1
                    /* TEXT */
                  )]),
                  _: 1
                })]),
                _: 1
              }, 8, ["modelValue"])]),
              _: 1
            }, 8, ["keywords"]),
            _createVNode(_component_SearchMarker, { keywords: ["image"] }, {
              default: _withCtx(() => [_createVNode(MkInput, {
                type: "url",
                modelValue: notFoundImageUrl.value,
                "onUpdate:modelValue": _cache[8] || (_cache[8] = ($event) => notFoundImageUrl.value = $event)
              }, {
                prefix: _withCtx(() => [_hoisted_6]),
                label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                  default: _withCtx(() => [_createTextVNode(
                    _toDisplayString(_unref(i18n).ts.notFoundDescription),
                    1
                    /* TEXT */
                  )]),
                  _: 1
                })]),
                _: 1
              }, 8, ["modelValue"])]),
              _: 1
            }, 8, ["keywords"]),
            _createVNode(_component_SearchMarker, { keywords: ["image"] }, {
              default: _withCtx(() => [_createVNode(MkInput, {
                type: "url",
                modelValue: infoImageUrl.value,
                "onUpdate:modelValue": _cache[9] || (_cache[9] = ($event) => infoImageUrl.value = $event)
              }, {
                prefix: _withCtx(() => [_hoisted_7]),
                label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                  default: _withCtx(() => [_createTextVNode(
                    _toDisplayString(_unref(i18n).ts.nothing),
                    1
                    /* TEXT */
                  )]),
                  _: 1
                })]),
                _: 1
              }, 8, ["modelValue"])]),
              _: 1
            }, 8, ["keywords"]),
            _createVNode(_component_SearchMarker, { keywords: ["image"] }, {
              default: _withCtx(() => [_createVNode(MkInput, {
                type: "url",
                modelValue: serverErrorImageUrl.value,
                "onUpdate:modelValue": _cache[10] || (_cache[10] = ($event) => serverErrorImageUrl.value = $event)
              }, {
                prefix: _withCtx(() => [_hoisted_8]),
                label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                  default: _withCtx(() => [_createTextVNode(
                    _toDisplayString(_unref(i18n).ts.somethingHappened),
                    1
                    /* TEXT */
                  )]),
                  _: 1
                })]),
                _: 1
              }, 8, ["modelValue"])]),
              _: 1
            }, 8, ["keywords"]),
            _createVNode(_component_SearchMarker, { keywords: ["theme", "color"] }, {
              default: _withCtx(() => [_createVNode(MkColorInput, {
                modelValue: themeColor.value,
                "onUpdate:modelValue": _cache[11] || (_cache[11] = ($event) => themeColor.value = $event)
              }, {
                label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                  default: _withCtx(() => [_createTextVNode(
                    _toDisplayString(_unref(i18n).ts.themeColor),
                    1
                    /* TEXT */
                  )]),
                  _: 1
                })]),
                _: 1
              }, 8, ["modelValue"])]),
              _: 1
            }, 8, ["keywords"]),
            _createVNode(_component_SearchMarker, { keywords: [
              "theme",
              "default",
              "light"
            ] }, {
              default: _withCtx(() => [_createVNode(MkTextarea, {
                modelValue: defaultLightTheme.value,
                "onUpdate:modelValue": _cache[12] || (_cache[12] = ($event) => defaultLightTheme.value = $event)
              }, {
                label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                  default: _withCtx(() => [_createTextVNode(
                    _toDisplayString(_unref(i18n).ts.instanceDefaultLightTheme),
                    1
                    /* TEXT */
                  )]),
                  _: 1
                })]),
                caption: _withCtx(() => [_createTextVNode(
                  _toDisplayString(_unref(i18n).ts.instanceDefaultThemeDescription),
                  1
                  /* TEXT */
                )]),
                _: 1
              }, 8, ["modelValue"])]),
              _: 1
            }, 8, ["keywords"]),
            _createVNode(_component_SearchMarker, { keywords: [
              "theme",
              "default",
              "dark"
            ] }, {
              default: _withCtx(() => [_createVNode(MkTextarea, {
                modelValue: defaultDarkTheme.value,
                "onUpdate:modelValue": _cache[13] || (_cache[13] = ($event) => defaultDarkTheme.value = $event)
              }, {
                label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                  default: _withCtx(() => [_createTextVNode(
                    _toDisplayString(_unref(i18n).ts.instanceDefaultDarkTheme),
                    1
                    /* TEXT */
                  )]),
                  _: 1
                })]),
                caption: _withCtx(() => [_createTextVNode(
                  _toDisplayString(_unref(i18n).ts.instanceDefaultThemeDescription),
                  1
                  /* TEXT */
                )]),
                _: 1
              }, 8, ["modelValue"])]),
              _: 1
            }, 8, ["keywords"]),
            _createVNode(_component_SearchMarker, null, {
              default: _withCtx(() => [_createVNode(MkInput, {
                type: "url",
                modelValue: repositoryUrl.value,
                "onUpdate:modelValue": _cache[14] || (_cache[14] = ($event) => repositoryUrl.value = $event)
              }, {
                prefix: _withCtx(() => [_hoisted_9]),
                label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                  default: _withCtx(() => [_createTextVNode(
                    _toDisplayString(_unref(i18n).ts.repositoryUrl),
                    1
                    /* TEXT */
                  )]),
                  _: 1
                })]),
                _: 1
              }, 8, ["modelValue"])]),
              _: 1
            }),
            _createVNode(_component_SearchMarker, null, {
              default: _withCtx(() => [_createVNode(MkInput, {
                type: "url",
                modelValue: feedbackUrl.value,
                "onUpdate:modelValue": _cache[15] || (_cache[15] = ($event) => feedbackUrl.value = $event)
              }, {
                prefix: _withCtx(() => [_hoisted_10]),
                label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                  default: _withCtx(() => [_createTextVNode(
                    _toDisplayString(_unref(i18n).ts.feedbackUrl),
                    1
                    /* TEXT */
                  )]),
                  _: 1
                })]),
                _: 1
              }, 8, ["modelValue"])]),
              _: 1
            }),
            _createVNode(_component_SearchMarker, null, {
              default: _withCtx(() => [_createVNode(MkTextarea, {
                modelValue: manifestJsonOverride.value,
                "onUpdate:modelValue": _cache[16] || (_cache[16] = ($event) => manifestJsonOverride.value = $event)
              }, {
                label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                  default: _withCtx(() => [_createTextVNode(
                    _toDisplayString(_unref(i18n).ts._serverSettings.manifestJsonOverride),
                    1
                    /* TEXT */
                  )]),
                  _: 1
                })]),
                _: 1
              }, 8, ["modelValue"])]),
              _: 1
            })
          ])]),
          _: 1
        }, 8, ["label", "keywords"])])]),
        _: 1
      }, 8, ["tabs"]);
    };
  }
};
