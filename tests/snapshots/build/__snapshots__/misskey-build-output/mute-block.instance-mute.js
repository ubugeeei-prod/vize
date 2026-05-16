import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue";
const _hoisted_1 = /* @__PURE__ */ _createElementVNode("br");
const _hoisted_2 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-device-floppy" });
import { ref, watch } from "vue";
import MkTextarea from "@/components/MkTextarea.vue";
import MkInfo from "@/components/MkInfo.vue";
import MkButton from "@/components/MkButton.vue";
import { ensureSignin } from "@/i.js";
import { misskeyApi } from "@/utility/misskey-api.js";
import { i18n } from "@/i18n.js";
export default {
  __name: "mute-block.instance-mute",
  setup(__props) {
    const $i = ensureSignin();
    const instanceMutes = ref($i.mutedInstances.join("\n"));
    const changed = ref(false);
    async function save() {
      let mutes = instanceMutes.value.trim().split("\n").map((el) => el.trim()).filter((el) => el);
      await misskeyApi("i/update", { mutedInstances: mutes });
      changed.value = false;
      // Refresh filtered list to signal to the user how they've been saved
      instanceMutes.value = mutes.join("\n");
    }
    watch(instanceMutes, () => {
      changed.value = true;
    });
    return (_ctx, _cache) => {
      return _openBlock(), _createElementBlock("div", { class: "_gaps_m" }, [
        _createVNode(MkInfo, null, {
          default: _withCtx(() => [_createTextVNode(
            _toDisplayString(_unref(i18n).ts._instanceMute.title),
            1
            /* TEXT */
          )]),
          _: 1
        }),
        _createVNode(MkTextarea, {
          modelValue: instanceMutes.value,
          "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event) => instanceMutes.value = $event)
        }, {
          label: _withCtx(() => [_createTextVNode(
            _toDisplayString(_unref(i18n).ts._instanceMute.heading),
            1
            /* TEXT */
          )]),
          caption: _withCtx(() => [
            _createTextVNode(
              _toDisplayString(_unref(i18n).ts._instanceMute.instanceMuteDescription),
              1
              /* TEXT */
            ),
            _hoisted_1,
            _createTextVNode(
              _toDisplayString(_unref(i18n).ts._instanceMute.instanceMuteDescription2),
              1
              /* TEXT */
            )
          ]),
          _: 1
        }, 8, ["modelValue"]),
        _createVNode(MkButton, {
          primary: "",
          disabled: !changed.value,
          onClick: _cache[1] || (_cache[1] = ($event) => save())
        }, {
          default: _withCtx(() => [
            _hoisted_2,
            _createTextVNode(" "),
            _createTextVNode(
              _toDisplayString(_unref(i18n).ts.save),
              1
              /* TEXT */
            )
          ]),
          _: 1
        }, 8, ["disabled"])
      ]);
    };
  }
};
