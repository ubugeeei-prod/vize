import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"


export default /*@__PURE__*/_defineComponent({
  __name: 'TimelineDomainBlocks',
  setup(__props) {

const { client } = useMasto()
const paginator = client.value.v1.domainBlocks.list()
async function unblock(domain: string) {
  await client.value.v1.domainBlocks.remove({ domain })
}

return (_ctx: any,_cache: any) => {
  const _component_CommonDropdownItem = _resolveComponent("CommonDropdownItem")
  const _component_CommonPaginator = _resolveComponent("CommonPaginator")

  return (_openBlock(), _createBlock(_component_CommonPaginator, { paginator: _unref(paginator) }, {
      default: _withCtx(({ item }) => [
        _createVNode(_component_CommonDropdownItem, { class: "!cursor-auto" }, {
          actions: _withCtx(() => [
            _createElementVNode("div", {
              "i-ri:lock-unlock-line": "",
              "text-primary": "",
              "cursor-pointer": "",
              onClick: _cache[0] || (_cache[0] = ($event: any) => (unblock(_ctx.item)))
            })
          ]),
          default: _withCtx(() => [
            _createTextVNode(_toDisplayString(item), 1 /* TEXT */),
            _createTextVNode("\n        ")
          ]),
          _: 1 /* STABLE */
        })
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["paginator"]))
}
}

})
