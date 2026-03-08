import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveDirective as _resolveDirective, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-palette" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-pencil" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-copy" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-clipboard" })
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-trash" })
const _hoisted_6 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-plus" })
import { ref, watch } from 'vue'
import MkButton from '@/components/MkButton.vue'
import * as os from '@/os.js'
import { i18n } from '@/i18n.js'
import { deepClone } from '@/utility/clone.js'
import MkCustomEmoji from '@/components/global/MkCustomEmoji.vue'
import MkEmoji from '@/components/global/MkEmoji.vue'
import MkFolder from '@/components/MkFolder.vue'
import MkDraggable from '@/components/MkDraggable.vue'
import { copyToClipboard } from '@/utility/copy-to-clipboard.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'emoji-palette.palette',
  props: {
    palette: { type: Object, required: true }
  },
  emits: ["updateEmojis", "updateName", "del"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
const emojis = ref<string[]>(deepClone(props.palette.emojis));
watch(emojis, () => {
	emit('updateEmojis', emojis.value);
}, { deep: true });
function remove(reaction: string, ev: PointerEvent) {
	os.popupMenu([{
		text: i18n.ts.remove,
		action: () => {
			emojis.value = emojis.value.filter(x => x !== reaction);
		},
	}], getHTMLElement(ev));
}
function pick(ev: PointerEvent) {
	os.pickEmoji(getHTMLElement(ev), {
		showPinned: false,
	}).then(it => {
		const emoji = it;
		if (!emojis.value.includes(emoji)) {
			emojis.value.push(emoji);
		}
	});
}
function getHTMLElement(ev: PointerEvent): HTMLElement {
	const target = ev.currentTarget ?? ev.target;
	return target as HTMLElement;
}
function rename() {
	os.inputText({
		title: i18n.ts.rename,
		default: props.palette.name,
	}).then(({ canceled, result: name }) => {
		if (canceled) return;
		if (name != null) {
			emit('updateName', name);
		}
	});
}
function copy() {
	copyToClipboard(emojis.value.join(' '));
}
function paste() {
	// TODO: validate
	navigator.clipboard.readText().then(text => {
		emojis.value = text.split(' ');
	});
}
function del(ev: PointerEvent) {
	os.popupMenu([{
		text: i18n.ts.delete,
		action: () => {
			emit('del');
		},
	}], ev.currentTarget ?? ev.target);
}

return (_ctx: any,_cache: any) => {
  const _directive_panel = _resolveDirective("panel")

  return (_openBlock(), _createBlock(MkFolder, { defaultOpen: true }, {
      icon: _withCtx(() => [
        _hoisted_1
      ]),
      label: _withCtx(() => [
        _createTextVNode(_toDisplayString(__props.palette.name === '' ? '(' + _unref(i18n).ts.noName + ')' : __props.palette.name), 1 /* TEXT */)
      ]),
      footer: _withCtx(() => [
        _createElementVNode("div", { class: "_buttons" }, [
          _createVNode(MkButton, { onClick: rename }, {
            default: _withCtx(() => [
              _hoisted_2,
              _createTextVNode(" "),
              _createTextVNode(_toDisplayString(_unref(i18n).ts.rename), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          }),
          _createVNode(MkButton, { onClick: copy }, {
            default: _withCtx(() => [
              _hoisted_3,
              _createTextVNode(" "),
              _createTextVNode(_toDisplayString(_unref(i18n).ts.copy), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          }),
          _createVNode(MkButton, {
            danger: "",
            onClick: paste
          }, {
            default: _withCtx(() => [
              _hoisted_4,
              _createTextVNode(" "),
              _createTextVNode(_toDisplayString(_unref(i18n).ts.paste), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          }),
          _createVNode(MkButton, {
            danger: "",
            iconOnly: "",
            style: "margin-left: auto;",
            onClick: del
          }, {
            default: _withCtx(() => [
              _hoisted_5
            ]),
            _: 1 /* STABLE */
          })
        ])
      ]),
      default: _withCtx(() => [
        _createElementVNode("div", null, [
          _createElementVNode("div", { style: "border-radius: 6px;" }, [
            _createVNode(MkDraggable, {
              modelValue: emojis.value.map((emoji) => ({
  	id: emoji,
  	emoji
  })),
              direction: "horizontal",
              class: _normalizeClass(_ctx.$style.emojis),
              group: "emojiPalettes",
              "onUpdate:modelValue": _cache[0] || (_cache[0] = v => emojis.value = v.map(x => x.emoji))
            }, {
              default: _withCtx(({ item }) => [
                _createElementVNode("button", {
                  class: _normalizeClass(["_button", _ctx.$style.emojisItem]),
                  onClick: _cache[1] || (_cache[1] = ($event: any) => (remove(_ctx.item.emoji, $event)))
                }, [
                  (item.emoji[0] === ':')
                    ? (_openBlock(), _createBlock(MkCustomEmoji, {
                      key: 0,
                      style: "pointer-events: none;",
                      name: item.emoji,
                      normal: true,
                      fallbackToImage: true
                    }, null, 8 /* PROPS */, ["name", "normal", "fallbackToImage"]))
                    : (_openBlock(), _createBlock(MkEmoji, {
                      key: 1,
                      style: "pointer-events: none;",
                      emoji: item.emoji,
                      normal: true
                    }, null, 8 /* PROPS */, ["emoji", "normal"]))
                ])
              ]),
              footer: _withCtx(() => [
                _createElementVNode("button", {
                  class: _normalizeClass(["_button", _ctx.$style.emojisAdd]),
                  onClick: pick
                }, [
                  _hoisted_6
                ])
              ]),
              _: 1 /* STABLE */
            }, 8 /* PROPS */, ["modelValue"])
          ]),
          _createElementVNode("div", {
            class: _normalizeClass(_ctx.$style.editorCaption)
          }, _toDisplayString(_unref(i18n).ts.reactionSettingDescription2), 1 /* TEXT */)
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["defaultOpen"]))
}
}

})
