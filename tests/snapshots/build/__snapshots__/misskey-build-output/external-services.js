import { withAsyncContext as _withAsyncContext } from "vue";
import { openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue";
const _hoisted_1 = { class: "_beta" };
const _hoisted_2 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-key" });
const _hoisted_3 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-key" });
import { ref, computed } from "vue";
import MkInput from "@/components/MkInput.vue";
import MkButton from "@/components/MkButton.vue";
import MkSwitch from "@/components/MkSwitch.vue";
import * as os from "@/os.js";
import { misskeyApi } from "@/utility/misskey-api.js";
import { fetchInstance } from "@/instance.js";
import { i18n } from "@/i18n.js";
import { definePage } from "@/page.js";
import MkFolder from "@/components/MkFolder.vue";
export default {
  __name: "external-services",
  async setup(__props) {
    let __temp, __restore;
    const meta = ([__temp, __restore] = _withAsyncContext(() => misskeyApi("admin/meta")), __temp = await __temp, __restore(), __temp);
    const deeplAuthKey = ref(meta.deeplAuthKey ?? "");
    const deeplIsPro = ref(meta.deeplIsPro);
    const googleAnalyticsMeasurementId = ref(meta.googleAnalyticsMeasurementId ?? "");
    function save_deepl() {
      os.apiWithDialog("admin/update-meta", {
        deeplAuthKey: deeplAuthKey.value,
        deeplIsPro: deeplIsPro.value
      }).then(() => {
        fetchInstance(true);
      });
    }
    function save_googleAnalytics() {
      os.apiWithDialog("admin/update-meta", { googleAnalyticsMeasurementId: googleAnalyticsMeasurementId.value }).then(() => {
        fetchInstance(true);
      });
    }
    const headerActions = computed(() => []);
    const headerTabs = computed(() => []);
    definePage(() => ({
      title: i18n.ts.externalServices,
      icon: "ti ti-link"
    }));
    return (_ctx, _cache) => {
      const _component_SearchLabel = _resolveComponent("SearchLabel");
      const _component_SearchMarker = _resolveComponent("SearchMarker");
      const _component_PageWithHeader = _resolveComponent("PageWithHeader");
      return _openBlock(), _createBlock(_component_PageWithHeader, {
        actions: headerActions.value,
        tabs: headerTabs.value
      }, {
        default: _withCtx(() => [_createElementVNode("div", {
          class: "_spacer",
          style: "--MI_SPACER-w: 700px; --MI_SPACER-min: 16px; --MI_SPACER-max: 32px;"
        }, [_createVNode(_component_SearchMarker, {
          path: "/admin/external-services",
          label: _unref(i18n).ts.externalServices,
          keywords: [
            "external",
            "services",
            "thirdparty"
          ],
          icon: "ti ti-link"
        }, {
          default: _withCtx(() => [_createElementVNode("div", { class: "_gaps_m" }, [_createVNode(_component_SearchMarker, null, {
            default: _withCtx((slotProps) => [_createVNode(MkFolder, { defaultOpen: slotProps.isParentOfTarget }, {
              label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                default: _withCtx(() => [_createTextVNode("Google Analytics")]),
                _: 1
              }), _createElementVNode(
                "span",
                _hoisted_1,
                _toDisplayString(_unref(i18n).ts.beta),
                1
                /* TEXT */
              )]),
              default: _withCtx(() => [_createElementVNode("div", { class: "_gaps_m" }, [_createVNode(_component_SearchMarker, null, {
                default: _withCtx(() => [_createVNode(MkInput, {
                  modelValue: googleAnalyticsMeasurementId.value,
                  "onUpdate:modelValue": ($event) => googleAnalyticsMeasurementId.value = $event
                }, {
                  prefix: _withCtx(() => [_hoisted_2]),
                  label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                    default: _withCtx(() => [_createTextVNode("Measurement ID")]),
                    _: 1
                  })]),
                  _: 1
                }, 8, ["modelValue", "onUpdate:modelValue"])]),
                _: 1
              }), _createVNode(MkButton, {
                primary: "",
                onClick: save_googleAnalytics
              }, {
                default: _withCtx(() => [_createTextVNode("Save")]),
                _: 1
              })])]),
              _: 1
            }, 8, ["defaultOpen"])]),
            _: 1
          }), _createVNode(_component_SearchMarker, null, {
            default: _withCtx((slotProps) => [_createVNode(MkFolder, { defaultOpen: slotProps.isParentOfTarget }, {
              label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                default: _withCtx(() => [_createTextVNode("DeepL Translation")]),
                _: 1
              })]),
              default: _withCtx(() => [_createElementVNode("div", { class: "_gaps_m" }, [
                _createVNode(_component_SearchMarker, null, {
                  default: _withCtx(() => [_createVNode(MkInput, {
                    modelValue: deeplAuthKey.value,
                    "onUpdate:modelValue": ($event) => deeplAuthKey.value = $event
                  }, {
                    prefix: _withCtx(() => [_hoisted_3]),
                    label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                      default: _withCtx(() => [_createTextVNode("Auth Key")]),
                      _: 1
                    })]),
                    _: 1
                  }, 8, ["modelValue", "onUpdate:modelValue"])]),
                  _: 1
                }),
                _createVNode(_component_SearchMarker, null, {
                  default: _withCtx(() => [_createVNode(MkSwitch, {
                    modelValue: deeplIsPro.value,
                    "onUpdate:modelValue": ($event) => deeplIsPro.value = $event
                  }, {
                    label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                      default: _withCtx(() => [_createTextVNode("Pro account")]),
                      _: 1
                    })]),
                    _: 1
                  }, 8, ["modelValue", "onUpdate:modelValue"])]),
                  _: 1
                }),
                _createVNode(MkButton, {
                  primary: "",
                  onClick: save_deepl
                }, {
                  default: _withCtx(() => [_createTextVNode("Save")]),
                  _: 1
                })
              ])]),
              _: 1
            }, 8, ["defaultOpen"])]),
            _: 1
          })])]),
          _: 1
        }, 8, ["label", "keywords"])])]),
        _: 1
      }, 8, ["actions", "tabs"]);
    };
  }
};
