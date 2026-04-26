import { withAsyncContext as _withAsyncContext } from "vue";
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue";
const _hoisted_1 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-key" });
const _hoisted_2 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-key" });
const _hoisted_3 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-check" });
import { ref, computed } from "vue";
import MkSwitch from "@/components/MkSwitch.vue";
import MkInput from "@/components/MkInput.vue";
import FormSplit from "@/components/form/split.vue";
import * as os from "@/os.js";
import { misskeyApi } from "@/utility/misskey-api.js";
import { fetchInstance } from "@/instance.js";
import { i18n } from "@/i18n.js";
import { definePage } from "@/page.js";
import MkButton from "@/components/MkButton.vue";
export default {
  __name: "object-storage",
  async setup(__props) {
    let __temp, __restore;
    const meta = ([__temp, __restore] = _withAsyncContext(() => misskeyApi("admin/meta")), __temp = await __temp, __restore(), __temp);
    const useObjectStorage = ref(meta.useObjectStorage);
    const objectStorageBaseUrl = ref(meta.objectStorageBaseUrl);
    const objectStorageBucket = ref(meta.objectStorageBucket);
    const objectStoragePrefix = ref(meta.objectStoragePrefix);
    const objectStorageEndpoint = ref(meta.objectStorageEndpoint);
    const objectStorageRegion = ref(meta.objectStorageRegion);
    const objectStoragePort = ref(meta.objectStoragePort);
    const objectStorageAccessKey = ref(meta.objectStorageAccessKey);
    const objectStorageSecretKey = ref(meta.objectStorageSecretKey);
    const objectStorageUseSSL = ref(meta.objectStorageUseSSL);
    const objectStorageUseProxy = ref(meta.objectStorageUseProxy);
    const objectStorageSetPublicRead = ref(meta.objectStorageSetPublicRead);
    const objectStorageS3ForcePathStyle = ref(meta.objectStorageS3ForcePathStyle);
    function save() {
      os.apiWithDialog("admin/update-meta", {
        useObjectStorage: useObjectStorage.value,
        objectStorageBaseUrl: objectStorageBaseUrl.value,
        objectStorageBucket: objectStorageBucket.value,
        objectStoragePrefix: objectStoragePrefix.value,
        objectStorageEndpoint: objectStorageEndpoint.value,
        objectStorageRegion: objectStorageRegion.value,
        objectStoragePort: objectStoragePort.value,
        objectStorageAccessKey: objectStorageAccessKey.value,
        objectStorageSecretKey: objectStorageSecretKey.value,
        objectStorageUseSSL: objectStorageUseSSL.value,
        objectStorageUseProxy: objectStorageUseProxy.value,
        objectStorageSetPublicRead: objectStorageSetPublicRead.value,
        objectStorageS3ForcePathStyle: objectStorageS3ForcePathStyle.value
      }).then(() => {
        fetchInstance(true);
      });
    }
    const headerTabs = computed(() => []);
    definePage(() => ({
      title: i18n.ts.objectStorage,
      icon: "ti ti-cloud"
    }));
    return (_ctx, _cache) => {
      const _component_SearchLabel = _resolveComponent("SearchLabel");
      const _component_SearchMarker = _resolveComponent("SearchMarker");
      const _component_SearchText = _resolveComponent("SearchText");
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
              _hoisted_3,
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
          path: "/admin/object-storage",
          label: _unref(i18n).ts.objectStorage,
          keywords: ["objectStorage"],
          icon: "ti ti-cloud"
        }, {
          default: _withCtx(() => [_createElementVNode("div", { class: "_gaps_m" }, [_createVNode(_component_SearchMarker, null, {
            default: _withCtx(() => [_createVNode(MkSwitch, {
              modelValue: useObjectStorage.value,
              "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event) => useObjectStorage.value = $event)
            }, {
              default: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                default: _withCtx(() => [_createTextVNode(
                  _toDisplayString(_unref(i18n).ts.useObjectStorage),
                  1
                  /* TEXT */
                )]),
                _: 1
              })]),
              _: 1
            }, 8, ["modelValue"])]),
            _: 1
          }), useObjectStorage.value ? (_openBlock(), _createElementBlock(
            _Fragment,
            { key: 0 },
            [
              _createVNode(_component_SearchMarker, null, {
                default: _withCtx(() => [_createVNode(MkInput, {
                  placeholder: "https://example.com",
                  type: "url",
                  modelValue: objectStorageBaseUrl.value,
                  "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event) => objectStorageBaseUrl.value = $event)
                }, {
                  label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                    default: _withCtx(() => [_createTextVNode(
                      _toDisplayString(_unref(i18n).ts.objectStorageBaseUrl),
                      1
                      /* TEXT */
                    )]),
                    _: 1
                  })]),
                  caption: _withCtx(() => [_createVNode(_component_SearchText, null, {
                    default: _withCtx(() => [_createTextVNode(
                      _toDisplayString(_unref(i18n).ts.objectStorageBaseUrlDesc),
                      1
                      /* TEXT */
                    )]),
                    _: 1
                  })]),
                  _: 1
                }, 8, ["placeholder", "modelValue"])]),
                _: 1
              }),
              _createVNode(_component_SearchMarker, null, {
                default: _withCtx(() => [_createVNode(MkInput, {
                  modelValue: objectStorageBucket.value,
                  "onUpdate:modelValue": _cache[2] || (_cache[2] = ($event) => objectStorageBucket.value = $event)
                }, {
                  label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                    default: _withCtx(() => [_createTextVNode(
                      _toDisplayString(_unref(i18n).ts.objectStorageBucket),
                      1
                      /* TEXT */
                    )]),
                    _: 1
                  })]),
                  caption: _withCtx(() => [_createVNode(_component_SearchText, null, {
                    default: _withCtx(() => [_createTextVNode(
                      _toDisplayString(_unref(i18n).ts.objectStorageBucketDesc),
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
                  modelValue: objectStoragePrefix.value,
                  "onUpdate:modelValue": _cache[3] || (_cache[3] = ($event) => objectStoragePrefix.value = $event)
                }, {
                  label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                    default: _withCtx(() => [_createTextVNode(
                      _toDisplayString(_unref(i18n).ts.objectStoragePrefix),
                      1
                      /* TEXT */
                    )]),
                    _: 1
                  })]),
                  caption: _withCtx(() => [_createVNode(_component_SearchText, null, {
                    default: _withCtx(() => [_createTextVNode(
                      _toDisplayString(_unref(i18n).ts.objectStoragePrefixDesc),
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
                  placeholder: "example.com",
                  modelValue: objectStorageEndpoint.value,
                  "onUpdate:modelValue": _cache[4] || (_cache[4] = ($event) => objectStorageEndpoint.value = $event)
                }, {
                  label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                    default: _withCtx(() => [_createTextVNode(
                      _toDisplayString(_unref(i18n).ts.objectStorageEndpoint),
                      1
                      /* TEXT */
                    )]),
                    _: 1
                  })]),
                  prefix: _withCtx(() => [_createTextVNode("https://")]),
                  caption: _withCtx(() => [_createVNode(_component_SearchText, null, {
                    default: _withCtx(() => [_createTextVNode(
                      _toDisplayString(_unref(i18n).ts.objectStorageEndpointDesc),
                      1
                      /* TEXT */
                    )]),
                    _: 1
                  })]),
                  _: 1
                }, 8, ["placeholder", "modelValue"])]),
                _: 1
              }),
              _createVNode(_component_SearchMarker, null, {
                default: _withCtx(() => [_createVNode(MkInput, {
                  modelValue: objectStorageRegion.value,
                  "onUpdate:modelValue": _cache[5] || (_cache[5] = ($event) => objectStorageRegion.value = $event)
                }, {
                  label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                    default: _withCtx(() => [_createTextVNode(
                      _toDisplayString(_unref(i18n).ts.objectStorageRegion),
                      1
                      /* TEXT */
                    )]),
                    _: 1
                  })]),
                  caption: _withCtx(() => [_createVNode(_component_SearchText, null, {
                    default: _withCtx(() => [_createTextVNode(
                      _toDisplayString(_unref(i18n).ts.objectStorageRegionDesc),
                      1
                      /* TEXT */
                    )]),
                    _: 1
                  })]),
                  _: 1
                }, 8, ["modelValue"])]),
                _: 1
              }),
              _createVNode(FormSplit, { minWidth: 280 }, {
                default: _withCtx(() => [_createVNode(_component_SearchMarker, null, {
                  default: _withCtx(() => [_createVNode(MkInput, {
                    modelValue: objectStorageAccessKey.value,
                    "onUpdate:modelValue": _cache[6] || (_cache[6] = ($event) => objectStorageAccessKey.value = $event)
                  }, {
                    prefix: _withCtx(() => [_hoisted_1]),
                    label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                      default: _withCtx(() => [_createTextVNode("Access key")]),
                      _: 1
                    })]),
                    _: 1
                  }, 8, ["modelValue"])]),
                  _: 1
                }), _createVNode(_component_SearchMarker, null, {
                  default: _withCtx(() => [_createVNode(MkInput, {
                    type: "password",
                    modelValue: objectStorageSecretKey.value,
                    "onUpdate:modelValue": _cache[7] || (_cache[7] = ($event) => objectStorageSecretKey.value = $event)
                  }, {
                    prefix: _withCtx(() => [_hoisted_2]),
                    label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                      default: _withCtx(() => [_createTextVNode("Secret key")]),
                      _: 1
                    })]),
                    _: 1
                  }, 8, ["modelValue"])]),
                  _: 1
                })]),
                _: 1
              }, 8, ["minWidth"]),
              _createVNode(_component_SearchMarker, null, {
                default: _withCtx(() => [_createVNode(MkSwitch, {
                  modelValue: objectStorageUseSSL.value,
                  "onUpdate:modelValue": _cache[8] || (_cache[8] = ($event) => objectStorageUseSSL.value = $event)
                }, {
                  label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                    default: _withCtx(() => [_createTextVNode(
                      _toDisplayString(_unref(i18n).ts.objectStorageUseSSL),
                      1
                      /* TEXT */
                    )]),
                    _: 1
                  })]),
                  caption: _withCtx(() => [_createVNode(_component_SearchText, null, {
                    default: _withCtx(() => [_createTextVNode(
                      _toDisplayString(_unref(i18n).ts.objectStorageUseSSLDesc),
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
                default: _withCtx(() => [_createVNode(MkSwitch, {
                  modelValue: objectStorageUseProxy.value,
                  "onUpdate:modelValue": _cache[9] || (_cache[9] = ($event) => objectStorageUseProxy.value = $event)
                }, {
                  label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                    default: _withCtx(() => [_createTextVNode(
                      _toDisplayString(_unref(i18n).ts.objectStorageUseProxy),
                      1
                      /* TEXT */
                    )]),
                    _: 1
                  })]),
                  caption: _withCtx(() => [_createVNode(_component_SearchText, null, {
                    default: _withCtx(() => [_createTextVNode(
                      _toDisplayString(_unref(i18n).ts.objectStorageUseProxyDesc),
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
                default: _withCtx(() => [_createVNode(MkSwitch, {
                  modelValue: objectStorageSetPublicRead.value,
                  "onUpdate:modelValue": _cache[10] || (_cache[10] = ($event) => objectStorageSetPublicRead.value = $event)
                }, {
                  label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                    default: _withCtx(() => [_createTextVNode(
                      _toDisplayString(_unref(i18n).ts.objectStorageSetPublicRead),
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
                default: _withCtx(() => [_createVNode(MkSwitch, {
                  modelValue: objectStorageS3ForcePathStyle.value,
                  "onUpdate:modelValue": _cache[11] || (_cache[11] = ($event) => objectStorageS3ForcePathStyle.value = $event)
                }, {
                  label: _withCtx(() => [_createVNode(_component_SearchLabel, null, {
                    default: _withCtx(() => [_createTextVNode("s3ForcePathStyle")]),
                    _: 1
                  })]),
                  caption: _withCtx(() => [_createVNode(_component_SearchText, null, {
                    default: _withCtx(() => [_createTextVNode(
                      _toDisplayString(_unref(i18n).ts.s3ForcePathStyleDesc),
                      1
                      /* TEXT */
                    )]),
                    _: 1
                  })]),
                  _: 1
                }, 8, ["modelValue"])]),
                _: 1
              })
            ],
            64
            /* STABLE_FRAGMENT */
          )) : _createCommentVNode("v-if", true)])]),
          _: 1
        }, 8, ["label", "keywords"])])]),
        _: 1
      }, 8, ["tabs"]);
    };
  }
};
