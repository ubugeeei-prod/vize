import { openBlock as _openBlock, createElementBlock as _createElementBlock, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, unref as _unref } from "vue";
const _hoisted_1 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-info-circle" });
import { i18n } from "@/i18n.js";
import { hideRestoreBackupSuggestion, restoreFromCloudBackup } from "@/preferences/utility.js";
export default {
  __name: "PreferenceRestore",
  setup(__props) {
    function restore() {
      restoreFromCloudBackup();
    }
    function skip() {
      hideRestoreBackupSuggestion();
    }
    return (_ctx, _cache) => {
      return _openBlock(), _createElementBlock(
        "div",
        { class: _normalizeClass(_ctx.$style.root) },
        [
          _createElementVNode(
            "span",
            { class: _normalizeClass(_ctx.$style.icon) },
            [_hoisted_1],
            2
            /* CLASS */
          ),
          _createElementVNode(
            "span",
            { class: _normalizeClass(_ctx.$style.title) },
            _toDisplayString(_unref(i18n).ts._preferencesBackup.backupFound),
            3
            /* TEXT, CLASS */
          ),
          _createElementVNode(
            "span",
            { class: _normalizeClass(_ctx.$style.body) },
            [
              _createElementVNode(
                "button",
                {
                  class: "_textButton",
                  onClick: restore
                },
                _toDisplayString(_unref(i18n).ts.restore),
                1
                /* TEXT */
              ),
              _createTextVNode(" | "),
              _createElementVNode(
                "button",
                {
                  class: "_textButton",
                  onClick: skip
                },
                _toDisplayString(_unref(i18n).ts.skip),
                1
                /* TEXT */
              )
            ],
            2
            /* CLASS */
          )
        ],
        2
        /* CLASS */
      );
    };
  }
};
