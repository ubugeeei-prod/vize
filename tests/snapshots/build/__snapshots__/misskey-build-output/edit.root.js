import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, resolveDirective as _resolveDirective, renderList as _renderList, toDisplayString as _toDisplayString, normalizeStyle as _normalizeStyle, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = { class: "name" }
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-x" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-plus" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-device-floppy" })
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-device-floppy" })
const _hoisted_6 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-trash" })
import { computed, watch, ref } from 'vue'
import * as Misskey from 'misskey-js'
import MkButton from '@/components/MkButton.vue'
import MkInput from '@/components/MkInput.vue'
import MkTextarea from '@/components/MkTextarea.vue'
import MkSwitch from '@/components/MkSwitch.vue'
import { selectFile } from '@/utility/drive.js'
import * as os from '@/os.js'
import { misskeyApi } from '@/utility/misskey-api.js'
import { i18n } from '@/i18n.js'
import { useRouter } from '@/router.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'edit.root',
  props: {
    post: { type: null, required: true }
  },
  setup(__props: any) {

const props = __props
const router = useRouter();
const files = ref(props.post?.files ?? []);
const description = ref(props.post?.description ?? null);
const title = ref(props.post?.title ?? '');
const isSensitive = ref(props.post?.isSensitive ?? false);
function chooseFile(evt: MouseEvent) {
	selectFile({
		anchorElement: evt.currentTarget ?? evt.target,
		multiple: true,
	}).then(selected => {
		files.value = files.value.concat(selected);
	});
}
function remove(file: NonNullable<Misskey.entities.GalleryPost['files']>[number]) {
	files.value = files.value.filter(f => f.id !== file.id);
}
async function save() {
	if (props.post != null) {
		await os.apiWithDialog('gallery/posts/update', {
			postId: props.post.id,
			title: title.value,
			description: description.value,
			fileIds: files.value.map(file => file.id),
			isSensitive: isSensitive.value,
		});
		router.push('/gallery/:postId', {
			params: {
				postId: props.post.id,
			},
		});
	} else {
		const created = await os.apiWithDialog('gallery/posts/create', {
			title: title.value,
			description: description.value,
			fileIds: files.value.map(file => file.id),
			isSensitive: isSensitive.value,
		});
		router.push('/gallery/:postId', {
			params: {
				postId: created.id,
			},
		});
	}
}
async function del() {
	if (props.post == null) return;
	const { canceled } = await os.confirm({
		type: 'warning',
		text: i18n.ts.deleteConfirm,
	});
	if (canceled) return;
	await os.apiWithDialog('gallery/posts/delete', {
		postId: props.post.id,
	});
	router.push('/gallery');
}
const headerActions = computed(() => []);
const headerTabs = computed(() => []);

return (_ctx: any,_cache: any) => {
  const _component_PageWithHeader = _resolveComponent("PageWithHeader")
  const _directive_tooltip = _resolveDirective("tooltip")

  return (_openBlock(), _createBlock(_component_PageWithHeader, {
      actions: headerActions.value,
      tabs: headerTabs.value
    }, {
      default: _withCtx(() => [
        _createElementVNode("div", {
          class: "_spacer",
          style: "--MI_SPACER-w: 800px; --MI_SPACER-min: 16px; --MI_SPACER-max: 32px;"
        }, [
          _createVNode(MkInput, {
            modelValue: title.value,
            "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((title).value = $event))
          }, {
            label: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(i18n).ts.title), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["modelValue"]),
          _createVNode(MkTextarea, {
            max: 500,
            modelValue: description.value,
            "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event: any) => ((description).value = $event))
          }, {
            label: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(i18n).ts.description), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["max", "modelValue"]),
          _createElementVNode("div", { class: "_gaps_s" }, [
            (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(files.value, (file) => {
              return (_openBlock(), _createElementBlock("div", {
                key: file.id,
                class: "wqugxsfx",
                style: _normalizeStyle({ backgroundImage: file ? `url(${ file.thumbnailUrl })` : '' })
              }, [
                _createElementVNode("div", _hoisted_1, _toDisplayString(file.name), 1 /* TEXT */),
                _createElementVNode("button", {
                  class: "remove _button",
                  onClick: _cache[2] || (_cache[2] = ($event: any) => (remove(file)))
                }, [
                  _hoisted_2
                ])
              ], 4 /* STYLE */))
            }), 128 /* KEYED_FRAGMENT */)),
            _createVNode(MkButton, {
              primary: "",
              onClick: chooseFile
            }, {
              default: _withCtx(() => [
                _hoisted_3,
                _createTextVNode(" "),
                _createTextVNode(_toDisplayString(_unref(i18n).ts.attachFile), 1 /* TEXT */)
              ]),
              _: 1 /* STABLE */
            })
          ]),
          _createVNode(MkSwitch, {
            modelValue: isSensitive.value,
            "onUpdate:modelValue": _cache[3] || (_cache[3] = ($event: any) => ((isSensitive).value = $event))
          }, {
            default: _withCtx(() => [
              _createTextVNode(_toDisplayString(_unref(i18n).ts.markAsSensitive), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["modelValue"]),
          _createElementVNode("div", { class: "_buttons" }, [
            (props.post != null)
              ? (_openBlock(), _createBlock(MkButton, {
                key: 0,
                primary: "",
                onClick: save
              }, {
                default: _withCtx(() => [
                  _hoisted_4,
                  _createTextVNode(" "),
                  _createTextVNode(_toDisplayString(_unref(i18n).ts.save), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }))
              : (_openBlock(), _createBlock(MkButton, {
                key: 1,
                primary: "",
                onClick: save
              }, {
                default: _withCtx(() => [
                  _hoisted_5,
                  _createTextVNode(" "),
                  _createTextVNode(_toDisplayString(_unref(i18n).ts.publish), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              })),
            (props.post != null)
              ? (_openBlock(), _createBlock(MkButton, {
                key: 0,
                danger: "",
                onClick: del
              }, {
                default: _withCtx(() => [
                  _hoisted_6,
                  _createTextVNode(" "),
                  _createTextVNode(_toDisplayString(_unref(i18n).ts.delete), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }))
              : _createCommentVNode("v-if", true)
          ])
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["actions", "tabs"]))
}
}

})
