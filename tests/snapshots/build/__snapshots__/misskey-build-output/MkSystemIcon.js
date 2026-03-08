import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, normalizeClass as _normalizeClass } from "vue"


export default /*@__PURE__*/_defineComponent({
  __name: 'MkSystemIcon',
  props: {
    type: { type: String, required: true }
  },
  setup(__props: any) {

const props = __props

return (_ctx: any,_cache: any) => {
  return (__props.type === 'info')
      ? (_openBlock(), _createElementBlock("svg", {
        key: 0,
        class: _normalizeClass([_ctx.$style.icon, _ctx.$style.info]),
        viewBox: "0 0 160 160"
      }, [ _createElementVNode("path", {
          d: "M80,108L80,72",
          pathLength: "1",
          class: _normalizeClass([_ctx.$style.line, _ctx.$style.animLine])
        }), _createElementVNode("path", {
          d: "M80,52L80,52",
          class: _normalizeClass([_ctx.$style.line, _ctx.$style.animFade])
        }), _createElementVNode("circle", {
          cx: "80",
          cy: "80",
          r: "56",
          pathLength: "1",
          class: _normalizeClass([_ctx.$style.line, _ctx.$style.animCircle])
        }) ]))
      : (__props.type === 'question')
        ? (_openBlock(), _createElementBlock("svg", {
          key: 1,
          class: _normalizeClass([_ctx.$style.icon, _ctx.$style.question]),
          viewBox: "0 0 160 160"
        }, [ _createElementVNode("path", {
            d: "M80,92L79.991,84C88.799,83.98 96,76.962 96,68C96,59.038 88.953,52 79.991,52C71.03,52 64,59.038 64,68",
            pathLength: "1",
            class: _normalizeClass([_ctx.$style.line, _ctx.$style.animLine])
          }), _createElementVNode("path", {
            d: "M80,108L80,108",
            class: _normalizeClass([_ctx.$style.line, _ctx.$style.animFade])
          }), _createElementVNode("circle", {
            cx: "80",
            cy: "80",
            r: "56",
            pathLength: "1",
            class: _normalizeClass([_ctx.$style.line, _ctx.$style.animCircle])
          }) ]))
      : (__props.type === 'success')
        ? (_openBlock(), _createElementBlock("svg", {
          key: 2,
          class: _normalizeClass([_ctx.$style.icon, _ctx.$style.success]),
          viewBox: "0 0 160 160"
        }, [ _createElementVNode("path", {
            d: "M62,80L74,92L98,68",
            pathLength: "1",
            class: _normalizeClass([_ctx.$style.line, _ctx.$style.animLine])
          }), _createElementVNode("circle", {
            cx: "80",
            cy: "80",
            r: "56",
            pathLength: "1",
            class: _normalizeClass([_ctx.$style.line, _ctx.$style.animCircle])
          }) ]))
      : (__props.type === 'warn')
        ? (_openBlock(), _createElementBlock("svg", {
          key: 3,
          class: _normalizeClass([_ctx.$style.icon, _ctx.$style.warn]),
          viewBox: "0 0 160 160"
        }, [ _createElementVNode("path", {
            d: "M80,64L80,88",
            pathLength: "1",
            class: _normalizeClass([_ctx.$style.line, _ctx.$style.animLine])
          }), _createElementVNode("path", {
            d: "M80,108L80,108",
            class: _normalizeClass([_ctx.$style.line, _ctx.$style.animFade])
          }), _createElementVNode("path", {
            d: "M92,28L144,116C148.709,124.65 144.083,135.82 136,136L24,136C15.917,135.82 11.291,124.65 16,116L68,28C73.498,19.945 86.771,19.945 92,28Z",
            pathLength: "1",
            class: _normalizeClass([_ctx.$style.line, _ctx.$style.animLine])
          }) ]))
      : (__props.type === 'error')
        ? (_openBlock(), _createElementBlock("svg", {
          key: 4,
          class: _normalizeClass([_ctx.$style.icon, _ctx.$style.error]),
          viewBox: "0 0 160 160"
        }, [ _createElementVNode("path", {
            d: "M63,63L96,96",
            pathLength: "1",
            style: "--duration:0.3s;",
            class: _normalizeClass([_ctx.$style.line, _ctx.$style.animLine])
          }), _createElementVNode("path", {
            d: "M96,63L63,96",
            pathLength: "1",
            style: "--duration:0.3s;--delay:0.2s;",
            class: _normalizeClass([_ctx.$style.line, _ctx.$style.animLine])
          }), _createElementVNode("circle", {
            cx: "80",
            cy: "80",
            r: "56",
            pathLength: "1",
            class: _normalizeClass([_ctx.$style.line, _ctx.$style.animCircle])
          }) ]))
      : (__props.type === 'waiting')
        ? (_openBlock(), _createElementBlock("svg", {
          key: 5,
          class: _normalizeClass([_ctx.$style.icon, _ctx.$style.waiting]),
          viewBox: "0 0 160 160"
        }, [ _createElementVNode("circle", {
            cx: "80",
            cy: "80",
            r: "56",
            pathLength: "1",
            class: _normalizeClass([_ctx.$style.line, _ctx.$style.animCircleWaiting])
          }), _createElementVNode("circle", {
            cx: "80",
            cy: "80",
            r: "56",
            style: "opacity: 0.25;",
            class: _normalizeClass([_ctx.$style.line])
          }) ]))
      : _createCommentVNode("v-if", true)
}
}

})
