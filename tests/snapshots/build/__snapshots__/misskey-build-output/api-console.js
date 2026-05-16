import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, withCtx as _withCtx } from "vue";
const _hoisted_1 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-send" });
import { ref, computed } from "vue";
import JSON5 from "json5";
import MkButton from "@/components/MkButton.vue";
import MkInput from "@/components/MkInput.vue";
import MkTextarea from "@/components/MkTextarea.vue";
import MkSwitch from "@/components/MkSwitch.vue";
import { misskeyApi } from "@/utility/misskey-api.js";
import { definePage } from "@/page.js";
export default {
  __name: "api-console",
  setup(__props) {
    const body = ref("{}");
    const endpoint = ref("");
    const endpoints = ref([]);
    const sending = ref(false);
    const res = ref("");
    const withCredential = ref(true);
    misskeyApi("endpoints").then((endpointResponse) => {
      endpoints.value = endpointResponse;
    });
    function send() {
      sending.value = true;
      const requestBody = JSON5.parse(body.value);
      misskeyApi(endpoint.value, requestBody, requestBody.i || (withCredential.value ? undefined : null)).then((resp) => {
        sending.value = false;
        res.value = JSON5.stringify(resp, null, 2);
      }, (err) => {
        sending.value = false;
        res.value = JSON5.stringify(err, null, 2);
      });
    }
    function onEndpointChange() {
      misskeyApi("endpoint", { endpoint: endpoint.value }, withCredential.value ? undefined : null).then((resp) => {
        if (resp == null) {
          body.value = "{}";
          return;
        }
        const endpointBody = {};
        for (const p of resp.params) {
          endpointBody[p.name] = p.type === "String" ? "" : p.type === "Number" ? 0 : p.type === "Boolean" ? false : p.type === "Array" ? [] : p.type === "Object" ? {} : null;
        }
        body.value = JSON5.stringify(endpointBody, null, 2);
      });
    }
    const headerActions = computed(() => []);
    const headerTabs = computed(() => []);
    definePage(() => ({
      title: "API console",
      icon: "ti ti-terminal-2"
    }));
    return (_ctx, _cache) => {
      const _component_MkEllipsis = _resolveComponent("MkEllipsis");
      const _component_PageWithHeader = _resolveComponent("PageWithHeader");
      return _openBlock(), _createBlock(_component_PageWithHeader, {
        actions: headerActions.value,
        tabs: headerTabs.value
      }, {
        default: _withCtx(() => [_createElementVNode("div", {
          class: "_spacer",
          style: "--MI_SPACER-w: 700px;"
        }, [_createElementVNode("div", { class: "_gaps_m" }, [_createElementVNode("div", { class: "_gaps_m" }, [
          _createVNode(MkInput, {
            datalist: endpoints.value,
            "onUpdate:modelValue": [($event) => onEndpointChange(), ($event) => endpoint.value = $event],
            modelValue: endpoint.value
          }, {
            label: _withCtx(() => [_createTextVNode("Endpoint")]),
            _: 1
          }, 8, ["datalist", "modelValue"]),
          _createVNode(MkTextarea, {
            code: "",
            modelValue: body.value,
            "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event) => body.value = $event)
          }, {
            label: _withCtx(() => [_createTextVNode("Params (JSON or JSON5)")]),
            _: 1
          }, 8, ["modelValue"]),
          _createVNode(MkSwitch, {
            modelValue: withCredential.value,
            "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event) => withCredential.value = $event)
          }, {
            default: _withCtx(() => [_createTextVNode("\n          With credential\n        ")]),
            _: 1
          }, 8, ["modelValue"]),
          _createVNode(MkButton, {
            primary: "",
            disabled: sending.value,
            onClick: send
          }, {
            default: _withCtx(() => [sending.value ? (_openBlock(), _createBlock(_component_MkEllipsis, { key: 0 })) : (_openBlock(), _createElementBlock(
              _Fragment,
              { key: 1 },
              [_hoisted_1, _createTextVNode(" Send")],
              64
              /* STABLE_FRAGMENT */
            ))]),
            _: 2
          }, 1032, ["disabled"])
        ]), res.value ? (_openBlock(), _createElementBlock("div", { key: 0 }, [_createVNode(MkTextarea, {
          code: "",
          readonly: "",
          tall: "",
          modelValue: res.value,
          "onUpdate:modelValue": _cache[2] || (_cache[2] = ($event) => res.value = $event)
        }, {
          label: _withCtx(() => [_createTextVNode("Response")]),
          _: 1
        }, 8, ["modelValue"])])) : _createCommentVNode("v-if", true)])])]),
        _: 1
      }, 8, ["actions", "tabs"]);
    };
  }
};
