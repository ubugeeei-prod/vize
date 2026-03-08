import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-camera" })
import { onUnmounted, ref } from 'vue'
import * as Misskey from 'misskey-js'
import { useWidgetPropsManager } from './widget.js'
import type { WidgetComponentEmits, WidgetComponentExpose, WidgetComponentProps } from './widget.js'
import type { FormWithDefault, GetFormResultType } from '@/utility/form.js'
import { useStream } from '@/stream.js'
import { getStaticImageUrl } from '@/utility/media-proxy.js'
import { misskeyApi } from '@/utility/misskey-api.js'
import MkContainer from '@/components/MkContainer.vue'
import { prefer } from '@/preferences.js'
import { i18n } from '@/i18n.js'

type WidgetProps = GetFormResultType<typeof widgetPropsDef>;
const name = 'photos';

export default /*@__PURE__*/_defineComponent({
  __name: 'WidgetPhotos',
  setup(__props: any, { expose: __expose, emit: __emit }) {

const emit = __emit
const props = __props
const widgetPropsDef = {
	showHeader: {
		type: 'boolean',
		label: i18n.ts._widgetOptions.showHeader,
		default: true,
	},
	transparent: {
		type: 'boolean',
		label: i18n.ts._widgetOptions.transparent,
		default: false,
	},
} satisfies FormWithDefault;
const { widgetProps, configure } = useWidgetPropsManager(name,
	widgetPropsDef,
	props,
	emit,
);
const connection = useStream().useChannel('main');
const images = ref<Misskey.entities.DriveFile[]>([]);
const fetching = ref(true);
function onDriveFileCreated(file: Misskey.entities.DriveFile) {
	if (/^image\/.+$/.test(file.type)) {
		images.value.unshift(file);
		if (images.value.length > 9) images.value.pop();
	}
}
const thumbnail = (image: Misskey.entities.DriveFile): string => {
	return prefer.s.disableShowingAnimatedImages
		? getStaticImageUrl(image.url)
		: image.thumbnailUrl ?? image.url;
};
misskeyApi('drive/stream', {
	type: 'image/*',
	limit: 9,
}).then(res => {
	images.value = res;
	fetching.value = false;
});
connection.on('driveFileCreated', onDriveFileCreated);
onUnmounted(() => {
	connection.dispose();
});
__expose({
	name,
	configure,
	id: props.widget ? props.widget.id : null,
})

return (_ctx: any,_cache: any) => {
  const _component_MkLoading = _resolveComponent("MkLoading")

  return (_openBlock(), _createBlock(MkContainer, {
      showHeader: _unref(widgetProps).showHeader,
      naked: _unref(widgetProps).transparent,
      class: _normalizeClass(["mkw-photos", _ctx.$style.root]),
      "data-transparent": _unref(widgetProps).transparent ? true : null,
      "data-cy-mkw-photos": ""
    }, {
      icon: _withCtx(() => [
        _hoisted_1
      ]),
      header: _withCtx(() => [
        _createTextVNode(_toDisplayString(_unref(i18n).ts._widgets.photos), 1 /* TEXT */)
      ]),
      default: _withCtx(() => [
        _createElementVNode("div", { class: "" }, [
          (fetching.value)
            ? (_openBlock(), _createBlock(_component_MkLoading, { key: 0 }))
            : (_openBlock(), _createElementBlock("div", {
              key: 1,
              class: _normalizeClass(_ctx.$style.stream)
            }, [
              (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(images.value, (image, i) => {
                return (_openBlock(), _createElementBlock("div", {
                  key: i,
                  class: _normalizeClass(_ctx.$style.img),
                  style: _normalizeStyle({ backgroundImage: `url(${thumbnail(image)})` })
                }, 4 /* STYLE */))
              }), 128 /* KEYED_FRAGMENT */))
            ]))
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["showHeader", "naked", "data-transparent"]))
}
}

})
