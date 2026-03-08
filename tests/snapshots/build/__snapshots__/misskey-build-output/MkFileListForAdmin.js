import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, resolveDirective as _resolveDirective, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = { style: "opacity: 0.7;" }
const _hoisted_2 = { style: "margin-right: 1em;" }
import * as Misskey from 'misskey-js'
import type { Paginator } from '@/utility/paginator.js'
import MkPagination from '@/components/MkPagination.vue'
import MkDriveFileThumbnail from '@/components/MkDriveFileThumbnail.vue'
import bytes from '@/filters/bytes.js'
import { i18n } from '@/i18n.js'
import { dateString } from '@/filters/date.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkFileListForAdmin',
  props: {
    paginator: { type: null, required: true },
    viewMode: { type: String, required: true }
  },
  setup(__props: any) {


return (_ctx: any,_cache: any) => {
  const _component_MkAcct = _resolveComponent("MkAcct")
  const _component_MkTime = _resolveComponent("MkTime")
  const _component_MkA = _resolveComponent("MkA")
  const _directive_tooltip = _resolveDirective("tooltip")

  return (_openBlock(), _createElementBlock("div", null, [ _createVNode(MkPagination, { paginator: __props.paginator }, {
        default: _withCtx(({ items }) => [
          _createElementVNode("div", {
            class: _normalizeClass({
  				[_ctx.$style.grid]: __props.viewMode === 'grid',
  				[_ctx.$style.list]: __props.viewMode === 'list',
  				'_gaps_s': __props.viewMode === 'list',
  			})
          }, [
            (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(items, (file) => {
              return (_openBlock(), _createBlock(_component_MkA, {
                key: file.id,
                to: `/admin/file/${file.id}`,
                class: _normalizeClass([_ctx.$style.file, '_button'])
              }, {
                default: _withCtx(() => [
                  (file.isSensitive)
                    ? (_openBlock(), _createElementBlock("div", {
                      key: 0,
                      class: _normalizeClass(_ctx.$style.sensitiveLabel)
                    }, _toDisplayString(_unref(i18n).ts.sensitive), 1 /* TEXT */))
                    : _createCommentVNode("v-if", true),
                  _createVNode(MkDriveFileThumbnail, {
                    class: _normalizeClass(_ctx.$style.thumbnail),
                    file: file,
                    fit: "contain",
                    highlightWhenSensitive: true
                  }, null, 8 /* PROPS */, ["file", "highlightWhenSensitive"]),
                  (__props.viewMode === 'list')
                    ? (_openBlock(), _createElementBlock("div", {
                      key: 0,
                      class: _normalizeClass(_ctx.$style.body)
                    }, [
                      _createElementVNode("div", null, [
                        _createElementVNode("small", _hoisted_1, _toDisplayString(file.name), 1 /* TEXT */)
                      ]),
                      _createElementVNode("div", null, [
                        (file.user)
                          ? (_openBlock(), _createBlock(_component_MkAcct, {
                            key: 0,
                            user: file.user
                          }, null, 8 /* PROPS */, ["user"]))
                          : (_openBlock(), _createElementBlock("div", { key: 1 }, _toDisplayString(_unref(i18n).ts.system), 1 /* TEXT */))
                      ]),
                      _createElementVNode("div", null, [
                        _createElementVNode("span", _hoisted_2, _toDisplayString(file.type), 1 /* TEXT */),
                        _createElementVNode("span", null, _toDisplayString(bytes(file.size)), 1 /* TEXT */)
                      ]),
                      _createElementVNode("div", null, [
                        _createElementVNode("span", null, [
                          _createTextVNode(_toDisplayString(_unref(i18n).ts.registeredDate) + ": ", 1 /* TEXT */),
                          _createVNode(_component_MkTime, {
                            time: file.createdAt,
                            mode: "detail"
                          }, null, 8 /* PROPS */, ["time"])
                        ])
                      ])
                    ]))
                    : _createCommentVNode("v-if", true)
                ]),
                _: 2 /* DYNAMIC */
              }, 1032 /* PROPS, DYNAMIC_SLOTS */, ["to"]))
            }), 128 /* KEYED_FRAGMENT */))
          ], 2 /* CLASS */)
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["paginator"]) ]))
}
}

})
