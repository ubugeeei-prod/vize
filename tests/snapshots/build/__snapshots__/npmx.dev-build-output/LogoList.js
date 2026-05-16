import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, resolveComponent as _resolveComponent, renderList as _renderList, normalizeStyle as _normalizeStyle } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("path", { d: "M5.62151 0C-1.8519 10.6931 -1.89574 27.2683 5.62151 37.9997H10.6709C3.15538 27.2683 3.19922 10.6931 10.6709 0H5.62151Z", fill: "currentColor" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("path", { d: "M5.04935 0H0C7.4734 10.6931 7.51725 27.2683 0 37.9997H5.04935C12.5648 27.2683 12.521 10.6931 5.04935 0Z", fill: "currentColor" })

type BaseItem = {
  name: string
  url: string
  normalisingIndent?: string
  logo:
    | string
    | {
        dark: string
        light: string
      }
}
type GroupItem = {
  name: string
  items: BaseItem[]
}

export default /*@__PURE__*/_defineComponent({
  __name: 'LogoList',
  props: {
    list: { type: Array, required: true }
  },
  setup(__props: any) {

const props = __props

return (_ctx: any,_cache: any) => {
  const _component_AboutLogoImg = _resolveComponent("AboutLogoImg")

  return (_openBlock(), _createElementBlock("ul", { class: "flex flex-wrap gap-4 md:gap-x-6 md:gap-y-4 list-none" }, [ (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(__props.list, (item) => {
        return (_openBlock(), _createElementBlock("li", { key: item.name }, [
          ('logo' in item)
            ? (_openBlock(), _createElementBlock("a", {
              key: 0,
              href: item.url,
              target: "_blank",
              rel: "noopener noreferrer",
              class: "relative flex items-center justify-center h-14 min-w-14 rounded-md hover:bg-fg/10 transition-colors p-1",
              style: _normalizeStyle({ paddingBlock: item.normalisingIndent }),
              "aria-label": item.name
            }, [
              _createVNode(_component_AboutLogoImg, {
                src: item.logo,
                alt: item.name
              }, null, 8 /* PROPS */, ["src", "alt"])
            ]))
            : (item.items)
              ? (_openBlock(), _createElementBlock("div", {
                key: 1,
                class: "relative flex items-center justify-center"
              }, [
                _createElementVNode("svg", {
                  width: "11",
                  height: "38",
                  viewBox: "0 0 11 38",
                  fill: "none",
                  xmlns: "http://www.w3.org/2000/svg",
                  "aria-hidden": "true"
                }, [
                  _hoisted_1
                ]),
                _createElementVNode("ul", { class: "flex items-center justify-center h-full gap-0.5 list-none" }, [
                  (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(item.items, (groupItem) => {
                    return (_openBlock(), _createElementBlock("li", { key: groupItem.name }, [
                      _createElementVNode("a", {
                        href: groupItem.url,
                        target: "_blank",
                        rel: "noopener noreferrer",
                        class: "relative flex items-center justify-center h-10 min-w-10 rounded-md hover:bg-fg/10 transition-colors p-0.5",
                        style: _normalizeStyle({ paddingBlock: groupItem.normalisingIndent }),
                        "aria-label": groupItem.name
                      }, [
                        _createVNode(_component_AboutLogoImg, {
                          src: groupItem.logo,
                          alt: groupItem.name
                        }, null, 8 /* PROPS */, ["src", "alt"])
                      ], 12 /* STYLE, PROPS */, ["href", "aria-label"])
                    ]))
                  }), 128 /* KEYED_FRAGMENT */))
                ]),
                _createElementVNode("svg", {
                  width: "11",
                  height: "38",
                  viewBox: "0 0 11 38",
                  fill: "none",
                  xmlns: "http://www.w3.org/2000/svg",
                  "aria-hidden": "true"
                }, [
                  _hoisted_2
                ])
              ]))
            : _createCommentVNode("v-if", true)
        ]))
      }), 128 /* KEYED_FRAGMENT */)) ]))
}
}

})
