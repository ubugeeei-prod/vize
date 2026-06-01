import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue";
import { ref, computed } from "vue";
import * as Misskey from "misskey-js";
import MkButton from "@/components/MkButton.vue";
import MkPostForm from "@/components/MkPostForm.vue";
import * as os from "@/os.js";
import { misskeyApi } from "@/utility/misskey-api.js";
import { definePage } from "@/page.js";
import { postMessageToParentWindow } from "@/utility/post-message.js";
import { i18n } from "@/i18n.js";
export default {
  __name: "share",
  setup(__props) {
    const urlParams = new URLSearchParams(window.location.search);
    const localOnlyQuery = urlParams.get("localOnly");
    const visibilityQuery = urlParams.get("visibility");
    const state = ref("fetching");
    const title = ref(urlParams.get("title"));
    const text = urlParams.get("text");
    const url = urlParams.get("url");
    const initialText = ref();
    const reply = ref();
    const renote = ref();
    const visibility = ref(Misskey.noteVisibilities.includes(visibilityQuery) ? visibilityQuery : undefined);
    const localOnly = ref(localOnlyQuery === "0" ? false : localOnlyQuery === "1" ? true : undefined);
    const files = ref([]);
    const visibleUsers = ref([]);
    async function init() {
      let noteText = "";
      if (title.value) {
        noteText += `[ ${title.value} ]\n`;
        //#region add text to note text
        if (text?.startsWith(title.value)) {
          // For the Google app https://github.com/misskey-dev/misskey/issues/16224
          noteText += text.replace(title.value, "").trimStart();
        } else if (text) {
          noteText += `${text}\n`;
        }
      } else if (text) {
        noteText += `${text}\n`;
      }
      if (url) {
        try {
          // Normalize the URL to URL-encoded and puny-coded from with the URL constructor.
          //
          // It's common to use unicode characters in the URL for better visibility of URL
          //     like: https://ja.wikipedia.org/wiki/ミスキー
          //  or like: https://藍.moe/
          // However, in the MFM, the unicode characters must be URL-encoded to be parsed as `url` node
          //     like: https://ja.wikipedia.org/wiki/%E3%83%9F%E3%82%B9%E3%82%AD%E3%83%BC
          //  or like: https://xn--931a.moe/
          // Therefore, we need to normalize the URL to URL-encoded form.
          //
          // The URL constructor will parse the URL and normalize unicode characters
          //   in the host to punycode and in the path component to URL-encoded form.
          //   (see url.spec.whatwg.org)
          //
          // In addition, the current MFM renderer decodes the URL-encoded path and / punycode encoded host name so
          //   this normalization doesn't make the visible URL ugly.
          //   (see MkUrl.vue)
          noteText += new URL(url).href;
        } catch {
          // fallback to original URL if the URL is invalid.
          // note that this is extremely rare since the `url` parameter is designed to share a URL and
          // the URL constructor will throw TypeError only if failure, which means the URL is not valid.
          noteText += url;
        }
      }
      initialText.value = noteText.trim();
      if (visibility.value === "specified") {
        const visibleUserIds = urlParams.get("visibleUserIds");
        const visibleAccts = urlParams.get("visibleAccts");
        await Promise.all([...visibleUserIds ? visibleUserIds.split(",").map((userId) => ({ userId })) : [], ...visibleAccts ? visibleAccts.split(",").map(Misskey.acct.parse) : []].map((q) => misskeyApi("users/show", q).then((user) => {
          visibleUsers.value.push(user);
        }, () => {
          console.error(`Invalid user query: ${JSON.stringify(q)}`);
        })));
      }
      try {
        //#region Reply
        const replyId = urlParams.get("replyId");
        const replyUri = urlParams.get("replyUri");
        if (replyId) {
          reply.value = await misskeyApi("notes/show", { noteId: replyId });
        } else if (replyUri) {
          const obj = await misskeyApi("ap/show", { uri: replyUri });
          if (obj.type === "Note") {
            reply.value = obj.object;
          }
        }
        //#endregion
        //#region Renote
        const renoteId = urlParams.get("renoteId");
        const renoteUri = urlParams.get("renoteUri");
        if (renoteId) {
          renote.value = await misskeyApi("notes/show", { noteId: renoteId });
        } else if (renoteUri) {
          const obj = await misskeyApi("ap/show", { uri: renoteUri });
          if (obj.type === "Note") {
            renote.value = obj.object;
          }
        }
        //#endregion
        //#region Drive files
        const fileIds = urlParams.get("fileIds");
        if (fileIds) {
          await Promise.all(fileIds.split(",").map((fileId) => misskeyApi("drive/files/show", { fileId }).then((file) => {
            files.value.push(file);
          }, () => {
            console.error(`Failed to fetch a file ${fileId}`);
          })));
        }
      } catch (err) {
        os.alert({
          type: "error",
          title: err.message,
          text: err.name
        });
      }
      state.value = "writing";
    }
    init();
    function close() {
      window.close();
      // 閉じなければ100ms後タイムラインに
      window.setTimeout(() => {
        window.location.href = "/";
      }, 100);
    }
    function goToMisskey() {
      window.location.href = "/";
    }
    function onPosted() {
      state.value = "posted";
      postMessageToParentWindow("misskey:shareForm:shareCompleted");
    }
    const headerActions = computed(() => []);
    const headerTabs = computed(() => []);
    definePage(() => ({
      title: i18n.ts.share,
      icon: "ti ti-share"
    }));
    return (_ctx, _cache) => {
      const _component_PageWithHeader = _resolveComponent("PageWithHeader");
      return _openBlock(), _createBlock(_component_PageWithHeader, {
        actions: headerActions.value,
        tabs: headerTabs.value
      }, {
        default: _withCtx(() => [_createElementVNode("div", {
          class: "_spacer",
          style: "--MI_SPACER-w: 800px;"
        }, [state.value === "writing" ? (_openBlock(), _createBlock(MkPostForm, {
          key: 0,
          fixed: "",
          instant: true,
          initialText: initialText.value,
          initialVisibility: visibility.value,
          initialFiles: files.value,
          initialLocalOnly: localOnly.value,
          reply: reply.value,
          renote: renote.value,
          initialVisibleUsers: visibleUsers.value,
          class: "_panel",
          onPosted
        }, null, 8, [
          "instant",
          "initialText",
          "initialVisibility",
          "initialFiles",
          "initialLocalOnly",
          "reply",
          "renote",
          "initialVisibleUsers"
        ])) : state.value === "posted" ? (_openBlock(), _createElementBlock("div", {
          key: 1,
          class: "_buttonsCenter"
        }, [_createVNode(MkButton, {
          primary: "",
          onClick: close
        }, {
          default: _withCtx(() => [_createTextVNode(
            _toDisplayString(_unref(i18n).ts.close),
            1
            /* TEXT */
          )]),
          _: 1
        }), _createVNode(MkButton, { onClick: goToMisskey }, {
          default: _withCtx(() => [_createTextVNode(
            _toDisplayString(_unref(i18n).ts.goToMisskey),
            1
            /* TEXT */
          )]),
          _: 1
        })])) : _createCommentVNode("v-if", true)])]),
        _: 1
      }, 8, ["actions", "tabs"]);
    };
  }
};
