import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue";
const _hoisted_1 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-plus" });
import { computed, onMounted, ref } from "vue";
import XItem from "./system-webhook.item.vue";
import FormSection from "@/components/form/section.vue";
import { definePage } from "@/page.js";
import { i18n } from "@/i18n.js";
import MkButton from "@/components/MkButton.vue";
import { misskeyApi } from "@/utility/misskey-api.js";
import { showSystemWebhookEditorDialog } from "@/components/MkSystemWebhookEditor.impl.js";
import * as os from "@/os.js";
export default {
  __name: "system-webhook",
  setup(__props) {
    const webhooks = ref([]);
    const headerActions = computed(() => []);
    const headerTabs = computed(() => []);
    async function onCreateWebhookClicked() {
      await showSystemWebhookEditorDialog({ mode: "create" });
      await fetchWebhooks();
    }
    async function onEditButtonClicked(webhook) {
      await showSystemWebhookEditorDialog({
        mode: "edit",
        id: webhook.id
      });
      await fetchWebhooks();
    }
    async function onDeleteButtonClicked(webhook) {
      const result = await os.confirm({
        type: "warning",
        title: i18n.ts._webhookSettings.deleteConfirm
      });
      if (!result.canceled) {
        await misskeyApi("admin/system-webhook/delete", { id: webhook.id });
        await fetchWebhooks();
      }
    }
    async function fetchWebhooks() {
      const result = await misskeyApi("admin/system-webhook/list", {});
      webhooks.value = result.sort((a, b) => a.id.localeCompare(b.id));
    }
    onMounted(async () => {
      await fetchWebhooks();
    });
    definePage(() => ({
      title: "SystemWebhook",
      icon: "ti ti-webhook"
    }));
    return (_ctx, _cache) => {
      const _component_SearchLabel = _resolveComponent("SearchLabel");
      const _component_SearchMarker = _resolveComponent("SearchMarker");
      const _component_PageWithHeader = _resolveComponent("PageWithHeader");
      return _openBlock(), _createBlock(_component_PageWithHeader, {
        actions: headerActions.value,
        tabs: headerTabs.value
      }, {
        default: _withCtx(() => [_createElementVNode("div", {
          class: "_spacer",
          style: "--MI_SPACER-w: 900px;"
        }, [_createVNode(_component_SearchMarker, {
          path: "/admin/system-webhook",
          label: "SystemWebhook",
          keywords: ["webhook"],
          icon: "ti ti-webhook"
        }, {
          default: _withCtx(() => [_createElementVNode("div", { class: "_gaps_m" }, [_createVNode(_component_SearchMarker, null, {
            default: _withCtx(() => [_createVNode(MkButton, {
              primary: "",
              onClick: onCreateWebhookClicked
            }, {
              default: _withCtx(() => [
                _hoisted_1,
                _createTextVNode(" "),
                _createVNode(_component_SearchLabel, null, {
                  default: _withCtx(() => [_createTextVNode(
                    _toDisplayString(_unref(i18n).ts._webhookSettings.createWebhook),
                    1
                    /* TEXT */
                  )]),
                  _: 1
                })
              ]),
              _: 1
            })]),
            _: 1
          }), _createVNode(FormSection, null, {
            default: _withCtx(() => [_createElementVNode("div", { class: "_gaps" }, [(_openBlock(true), _createElementBlock(
              _Fragment,
              null,
              _renderList(webhooks.value, (item) => {
                return _openBlock(), _createBlock(XItem, {
                  key: item.id,
                  entity: item,
                  onEdit: onEditButtonClicked,
                  onDelete: onDeleteButtonClicked
                }, null, 8, ["entity"]);
              }),
              128
              /* KEYED_FRAGMENT */
            ))])]),
            _: 1
          })])]),
          _: 1
        }, 8, ["keywords"])])]),
        _: 1
      }, 8, ["actions", "tabs"]);
    };
  }
};
