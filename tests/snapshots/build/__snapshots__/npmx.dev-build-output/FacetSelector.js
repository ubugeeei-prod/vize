import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = { class: "text-3xs text-fg-subtle uppercase tracking-wider" }
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("span", { class: "text-2xs text-fg-muted/40" }, "/")

export default /*@__PURE__*/_defineComponent({
  __name: 'FacetSelector',
  setup(__props) {

const {
  isFacetSelected,
  toggleFacet,
  selectCategory,
  deselectCategory,
  facetsByCategory,
  categoryOrder,
  getCategoryLabel,
} = useFacetSelection()
// Check if all non-comingSoon facets in a category are selected
function isCategoryAllSelected(category: string): boolean {
  const facets = facetsByCategory.value[category] ?? []
  const selectableFacets = facets.filter(f => !f.comingSoon)
  return selectableFacets.length > 0 && selectableFacets.every(f => isFacetSelected(f.id))
}
// Check if no facets in a category are selected
function isCategoryNoneSelected(category: string): boolean {
  const facets = facetsByCategory.value[category] ?? []
  const selectableFacets = facets.filter(f => !f.comingSoon)
  return selectableFacets.length > 0 && selectableFacets.every(f => !isFacetSelected(f.id))
}

return (_ctx: any,_cache: any) => {
  const _component_ButtonBase = _resolveComponent("ButtonBase")

  return (_openBlock(), _createElementBlock("div", {
      class: "space-y-3",
      role: "group",
      "aria-label": _ctx.$t('compare.facets.group_label')
    }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_unref(categoryOrder), (category) => {
        return (_openBlock(), _createElementBlock("div", { key: category }, [
          _createElementVNode("div", { class: "flex items-center gap-2 mb-2" }, [
            _createElementVNode("span", _hoisted_1, _toDisplayString(_unref(getCategoryLabel)(category)), 1 /* TEXT */),
            _createTextVNode("\n        " + "\n        "),
            _createVNode(_component_ButtonBase, {
              "aria-label": 
              _ctx.$t('compare.facets.select_category', { category: _unref(getCategoryLabel)(category) })
            ,
              "aria-pressed": isCategoryAllSelected(category),
              disabled: isCategoryAllSelected(category),
              onClick: _cache[0] || (_cache[0] = ($event: any) => (_unref(selectCategory)(category))),
              size: "small"
            }, {
              default: _withCtx(() => [
                _createTextVNode(_toDisplayString(_ctx.$t('compare.facets.all')), 1 /* TEXT */)
              ]),
              _: 2 /* DYNAMIC */
            }, 8 /* PROPS */, ["aria-label", "aria-pressed", "disabled"]),
            _hoisted_2,
            _createVNode(_component_ButtonBase, {
              "aria-label": 
              _ctx.$t('compare.facets.deselect_category', { category: _unref(getCategoryLabel)(category) })
            ,
              "aria-pressed": isCategoryNoneSelected(category),
              disabled: isCategoryNoneSelected(category),
              onClick: _cache[1] || (_cache[1] = ($event: any) => (_unref(deselectCategory)(category))),
              size: "small"
            }, {
              default: _withCtx(() => [
                _createTextVNode(_toDisplayString(_ctx.$t('compare.facets.none')), 1 /* TEXT */)
              ]),
              _: 2 /* DYNAMIC */
            }, 8 /* PROPS */, ["aria-label", "aria-pressed", "disabled"])
          ]),
          _createTextVNode("\n\n      " + "\n      "),
          _createElementVNode("div", {
            class: "flex items-center gap-1.5 flex-wrap",
            role: "group"
          }, [
            (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_unref(facetsByCategory)[category], (facet) => {
              return (_openBlock(), _createBlock(_component_ButtonBase, {
                key: facet.id,
                size: "small",
                title: facet.comingSoon ? _ctx.$t('compare.facets.coming_soon') : facet.description,
                disabled: facet.comingSoon,
                "aria-pressed": _unref(isFacetSelected)(facet.id),
                "aria-label": facet.label,
                class: _normalizeClass(["gap-1 px-1.5 rounded transition-colors focus-visible:outline-accent/70", 
              facet.comingSoon
                ? 'text-fg-subtle/50 bg-bg-subtle border-border-subtle cursor-not-allowed'
                : _unref(isFacetSelected)(facet.id)
                  ? 'text-fg-muted bg-bg-muted'
                  : 'text-fg-subtle bg-bg-subtle border-border-subtle hover:text-fg-muted hover:border-border'
            ]),
                onClick: _cache[2] || (_cache[2] = ($event: any) => (!facet.comingSoon && _unref(toggleFacet)(facet.id))),
                classicon: 
              facet.comingSoon
                ? undefined
                : _unref(isFacetSelected)(facet.id)
                  ? 'i-lucide:check'
                  : 'i-lucide:plus'
          
              }, {
                default: _withCtx(() => [
                  _createTextVNode(_toDisplayString(facet.label), 1 /* TEXT */),
                  _createTextVNode("\n          "),
                  (facet.comingSoon)
                    ? (_openBlock(), _createElementBlock("span", {
                      key: 0,
                      class: "text-4xs"
                    }, "(" + _toDisplayString(_ctx.$t('compare.facets.coming_soon')) + ")", 1 /* TEXT */))
                    : _createCommentVNode("v-if", true)
                ]),
                _: 2 /* DYNAMIC */
              }, 1034 /* CLASS, PROPS, DYNAMIC_SLOTS */, ["title", "disabled", "aria-pressed", "aria-label", "classicon"]))
            }), 128 /* KEYED_FRAGMENT */))
          ])
        ]))
      }), 128 /* KEYED_FRAGMENT */)) ], 8 /* PROPS */, ["aria-label"]))
}
}

})
