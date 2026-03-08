import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, toDisplayString as _toDisplayString, normalizeStyle as _normalizeStyle, unref as _unref } from "vue"

import { onMounted, ref, useTemplateRef } from 'vue'
import * as Misskey from 'misskey-js'
import { useInterval } from '@@/js/use-interval.js'
import { useWidgetPropsManager } from './widget.js'
import type { WidgetComponentEmits, WidgetComponentExpose, WidgetComponentProps } from './widget.js'
import type { FormWithDefault, GetFormResultType } from '@/utility/form.js'
import * as os from '@/os.js'
import { misskeyApi } from '@/utility/misskey-api.js'
import { i18n } from '@/i18n.js'
import { selectDriveFolder } from '@/utility/drive.js'

type WidgetProps = GetFormResultType<typeof widgetPropsDef>;
const name = 'slideshow';

export default /*@__PURE__*/_defineComponent({
  __name: 'WidgetSlideshow',
  setup(__props: any, { expose: __expose, emit: __emit }) {

const emit = __emit
const props = __props
const widgetPropsDef = {
	height: {
		type: 'number',
		label: i18n.ts._widgetOptions.height,
		default: 300,
	},
	folderId: {
		type: 'string',
		default: null as string | null,
		hidden: true,
	},
} satisfies FormWithDefault;
const { widgetProps, configure, save } = useWidgetPropsManager(name,
	widgetPropsDef,
	props,
	emit,
);
const images = ref<Misskey.entities.DriveFile[]>([]);
const fetching = ref(true);
const slideA = useTemplateRef('slideA');
const slideB = useTemplateRef('slideB');
const change = () => {
	if (images.value.length === 0 || slideA.value == null || slideB.value == null) return;

	const index = Math.floor(Math.random() * images.value.length);
	const img = `url(${ images.value[index].url })`;

	slideB.value.style.backgroundImage = img;

	slideB.value.classList.add('anime');
	window.setTimeout(() => {
		// 既にこのウィジェットがunmountされていたら要素がない
		if (slideA.value == null) return;

		slideA.value.style.backgroundImage = img;

		slideB.value!.classList.remove('anime');
	}, 1000);
};
const fetch = () => {
	if (slideA.value == null || slideB.value == null) return;
	fetching.value = true;

	misskeyApi('drive/files', {
		folderId: widgetProps.folderId,
		type: 'image/*',
		limit: 100,
	}).then(res => {
		images.value = res;
		fetching.value = false;
		slideA.value!.style.backgroundImage = '';
		slideB.value!.style.backgroundImage = '';
		change();
	});
};
const choose = () => {
	selectDriveFolder(null).then(({ folders, canceled }) => {
		if (canceled || folders[0] == null) {
			return;
		}
		widgetProps.folderId = folders[0].id;
		save();
		fetch();
	});
};
useInterval(change, 10000, {
	immediate: false,
	afterMounted: true,
});
onMounted(() => {
	if (widgetProps.folderId != null) {
		fetch();
	}
});
__expose({
	name,
	configure,
	id: props.widget ? props.widget.id : null,
})

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", {
      "data-cy-mkw-slideshow": "",
      class: "kvausudm _panel mkw-slideshow",
      style: _normalizeStyle({ height: _unref(widgetProps).height + 'px' })
    }, [ _createElementVNode("div", { onClick: choose }, [ (_unref(widgetProps).folderId == null) ? (_openBlock(), _createElementBlock("p", { key: 0 }, _toDisplayString(_unref(i18n).ts.folder), 1 /* TEXT */)) : _createCommentVNode("v-if", true), (_unref(widgetProps).folderId != null && images.value.length === 0 && !fetching.value) ? (_openBlock(), _createElementBlock("p", { key: 0 }, _toDisplayString(_unref(i18n).ts.nothing), 1 /* TEXT */)) : _createCommentVNode("v-if", true), _createElementVNode("div", {
          ref_key: "slideA", ref: slideA,
          class: "slide a"
        }, null, 512 /* NEED_PATCH */), _createElementVNode("div", {
          ref_key: "slideB", ref: slideB,
          class: "slide b"
        }, null, 512 /* NEED_PATCH */) ]) ], 4 /* STYLE */))
}
}

})
