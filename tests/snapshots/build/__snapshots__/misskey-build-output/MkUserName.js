import { openBlock as _openBlock, createBlock as _createBlock, resolveComponent as _resolveComponent } from "vue";
export default {
  __name: "MkUserName",
  props: {
    user: {
      type: null,
      required: true
    },
    nowrap: {
      type: Boolean,
      required: false,
      default: true
    }
  },
  setup(__props) {
    const props = __props;
    return (_ctx, _cache) => {
      const _component_Mfm = _resolveComponent("Mfm");
      return _openBlock(), _createBlock(_component_Mfm, {
        text: __props.user.name ?? __props.user.username,
        author: __props.user,
        plain: true,
        nowrap: __props.nowrap,
        emojiUrls: __props.user.emojis
      }, null, 8, [
        "text",
        "author",
        "plain",
        "nowrap",
        "emojiUrls"
      ]);
    };
  }
};
