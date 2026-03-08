import { withAsyncContext as _withAsyncContext, defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"

import { computed } from 'vue'
import MkButton from '@/components/MkButton.vue'
import MkFolder from '@/components/MkFolder.vue'
import { i18n } from '@/i18n.js'
import { definePage } from '@/page.js'
import { deleteCloudBackup, listCloudBackups } from '@/preferences/utility.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'profiles',
  async setup(__props) {

let __temp: any, __restore: any

const backups = await listCloudBackups();
function del(backup: { name: string }): void {
	deleteCloudBackup(backup.name);
}
const headerActions = computed(() => []);
const headerTabs = computed(() => []);
definePage(() => ({
	title: i18n.ts._preferencesProfile.manageProfiles,
	icon: 'ti ti-settings-cog',
}));

return (_ctx: any,_cache: any) => {
  const _component_SearchMarker = _resolveComponent("SearchMarker")

  return (_openBlock(), _createBlock(_component_SearchMarker, {
      path: "/settings/profiles",
      label: _unref(i18n).ts._preferencesProfile.manageProfiles,
      keywords: ['profile', 'settings', 'preferences', 'manage'],
      icon: "ti ti-settings-cog"
    }, {
      default: _withCtx(() => [
        _createElementVNode("div", { class: "_gaps" }, [
          (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_unref(backups), (backup) => {
            return (_openBlock(), _createBlock(MkFolder, null, {
              label: _withCtx(() => [
                _createTextVNode(_toDisplayString(backup.name), 1 /* TEXT */)
              ]),
              default: _withCtx(() => [
                _createVNode(MkButton, {
                  danger: "",
                  onClick: _cache[0] || (_cache[0] = ($event: any) => (del(backup)))
                }, {
                  default: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_unref(i18n).ts.delete), 1 /* TEXT */)
                  ]),
                  _: 2 /* DYNAMIC */
                })
              ]),
              _: 2 /* DYNAMIC */
            }, 1024 /* DYNAMIC_SLOTS */))
          }), 256 /* UNKEYED_FRAGMENT */))
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["label", "keywords"]))
}
}

})
