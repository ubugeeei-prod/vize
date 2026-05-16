import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx } from "vue";
import { ref } from "vue";
import * as config from "@@/js/config.js";
import MkButton from "@/components/MkButton.vue";
import MkInput from "@/components/MkInput.vue";
import MkSwitch from "@/components/MkSwitch.vue";
import * as os from "@/os.js";
import { $i } from "@/i.js";
import { chooseDriveFile } from "@/utility/drive.js";
export default {
  __name: "MkPreview",
  setup(__props) {
    const text = ref("");
    const flag = ref(true);
    const mfm = ref(`Hello world! This is an @example mention. BTW you are @${$i ? $i.username : "guest"}.\nAlso, here is ${config.url} and [example link](${config.url}). for more details, see https://example.com.\nAs you know #misskey is open-source software.`);
    const openDialog = async () => {
      await os.alert({
        type: "warning",
        title: "Oh my Aichan",
        text: "Lorem ipsum dolor sit amet, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua."
      });
    };
    const openForm = async () => {
      await os.form("Example form", {
        foo: {
          type: "boolean",
          default: true,
          label: "This is a boolean property"
        },
        bar: {
          type: "number",
          default: 300,
          label: "This is a number property"
        },
        baz: {
          type: "string",
          default: "Misskey makes you happy.",
          label: "This is a string property"
        }
      });
    };
    const openDrive = async () => {
      await chooseDriveFile({ multiple: false });
    };
    const selectUser = async () => {
      await os.selectUser();
    };
    const openMenu = async (ev) => {
      os.popupMenu([
        {
          type: "label",
          text: "Fruits"
        },
        {
          text: "Create some apples",
          action: () => {}
        },
        {
          text: "Read some oranges",
          action: () => {}
        },
        {
          text: "Update some melons",
          action: () => {}
        },
        {
          text: "Delete some bananas",
          danger: true,
          action: () => {}
        }
      ], ev.currentTarget ?? ev.target);
    };
    return (_ctx, _cache) => {
      const _component_Mfm = _resolveComponent("Mfm");
      return _openBlock(), _createElementBlock(
        "div",
        { class: _normalizeClass(_ctx.$style.preview) },
        [
          _createElementVNode("div", null, [
            _createVNode(MkInput, {
              modelValue: text.value,
              "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event) => text.value = $event)
            }, {
              label: _withCtx(() => [_createTextVNode("Text")]),
              _: 1
            }, 8, ["modelValue"]),
            _createVNode(MkSwitch, {
              class: _normalizeClass(_ctx.$style.preview__content1__switch_button),
              modelValue: flag.value,
              "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event) => flag.value = $event)
            }, {
              default: _withCtx(() => [_createElementVNode(
                "span",
                null,
                "Switch is now " + _toDisplayString(flag.value ? "on" : "off"),
                1
                /* TEXT */
              )]),
              _: 1
            }, 10, ["modelValue"]),
            _createElementVNode(
              "div",
              { class: _normalizeClass(_ctx.$style.preview__content1__button) },
              [_createVNode(MkButton, { inline: "" }, {
                default: _withCtx(() => [_createTextVNode("This is")]),
                _: 1
              }), _createVNode(MkButton, {
                inline: "",
                primary: ""
              }, {
                default: _withCtx(() => [_createTextVNode("the button")]),
                _: 1
              })],
              2
              /* CLASS */
            )
          ]),
          _createElementVNode(
            "div",
            {
              class: _normalizeClass(_ctx.$style.preview__content2),
              style: "pointer-events: none;"
            },
            [_createVNode(_component_Mfm, { text: mfm.value }, null, 8, ["text"])],
            2
            /* CLASS */
          ),
          _createElementVNode(
            "div",
            { class: _normalizeClass(_ctx.$style.preview__content3) },
            [
              _createVNode(MkButton, {
                inline: "",
                primary: "",
                onClick: openMenu
              }, {
                default: _withCtx(() => [_createTextVNode("Open menu")]),
                _: 1
              }),
              _createVNode(MkButton, {
                inline: "",
                primary: "",
                onClick: openDialog
              }, {
                default: _withCtx(() => [_createTextVNode("Open dialog")]),
                _: 1
              }),
              _createVNode(MkButton, {
                inline: "",
                primary: "",
                onClick: openForm
              }, {
                default: _withCtx(() => [_createTextVNode("Open form")]),
                _: 1
              }),
              _createVNode(MkButton, {
                inline: "",
                primary: "",
                onClick: openDrive
              }, {
                default: _withCtx(() => [_createTextVNode("Open drive")]),
                _: 1
              })
            ],
            2
            /* CLASS */
          )
        ],
        2
        /* CLASS */
      );
    };
  }
};
