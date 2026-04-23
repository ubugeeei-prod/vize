import { withAsyncContext as _withAsyncContext } from "vue";
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, renderList as _renderList, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue";
import { useTemplateRef, computed } from "vue";
import { notificationTypes } from "misskey-js";
import XNotificationConfig from "./notifications.notification-config.vue";
import FormLink from "@/components/form/link.vue";
import FormSection from "@/components/form/section.vue";
import MkFolder from "@/components/MkFolder.vue";
import MkSwitch from "@/components/MkSwitch.vue";
import MkButton from "@/components/MkButton.vue";
import * as os from "@/os.js";
import { ensureSignin } from "@/i.js";
import { misskeyApi } from "@/utility/misskey-api.js";
import { i18n } from "@/i18n.js";
import { definePage } from "@/page.js";
import MkPushNotificationAllowButton from "@/components/MkPushNotificationAllowButton.vue";
import MkFeatureBanner from "@/components/MkFeatureBanner.vue";
export default {
  __name: "notifications",
  async setup(__props) {
    let __temp, __restore;
    const $i = ensureSignin();
    const nonConfigurableNotificationTypes = [
      "note",
      "roleAssigned",
      "followRequestAccepted",
      "test",
      "exportCompleted"
    ];
    const configurableNotificationTypes = notificationTypes.filter((type) => !nonConfigurableNotificationTypes.includes(type));
    const onlyOnOrOffNotificationTypes = [
      "app",
      "achievementEarned",
      "login",
      "createToken",
      "scheduledNotePosted",
      "scheduledNotePostFailed"
    ];
    const allowButton = useTemplateRef("allowButton");
    const pushRegistrationInServer = computed(() => allowButton.value?.pushRegistrationInServer);
    const sendReadMessage = computed(() => pushRegistrationInServer.value?.sendReadMessage || false);
    const userLists = ([__temp, __restore] = _withAsyncContext(() => misskeyApi("users/lists/list")), __temp = await __temp, __restore(), __temp);
    async function readAllNotifications() {
      await os.apiWithDialog("notifications/mark-all-as-read", {});
    }
    async function updateReceiveConfig(type, value) {
      await os.apiWithDialog("i/update", { notificationRecieveConfig: {
        ...$i.notificationRecieveConfig,
        [type]: value
      } }).then((i) => {
        $i.notificationRecieveConfig = i.notificationRecieveConfig;
      });
    }
    function onChangeSendReadMessage(v) {
      if (!pushRegistrationInServer.value) return;
      os.apiWithDialog("sw/update-registration", {
        endpoint: pushRegistrationInServer.value.endpoint,
        sendReadMessage: v
      }).then((res) => {
        if (!allowButton.value) return;
        allowButton.value.pushRegistrationInServer = res;
      });
    }
    function testNotification() {
      misskeyApi("notifications/test-notification");
    }
    async function flushNotification() {
      const { canceled } = await os.confirm({
        type: "warning",
        text: i18n.ts.resetAreYouSure
      });
      if (canceled) return;
      os.apiWithDialog("notifications/flush", {});
    }
    const headerActions = computed(() => []);
    const headerTabs = computed(() => []);
    definePage(() => ({
      title: i18n.ts.notifications,
      icon: "ti ti-bell"
    }));
    return (_ctx, _cache) => {
      const _component_SearchText = _resolveComponent("SearchText");
      const _component_SearchMarker = _resolveComponent("SearchMarker");
      return _openBlock(), _createBlock(_component_SearchMarker, {
        path: "/settings/notifications",
        label: _unref(i18n).ts.notifications,
        keywords: ["notifications"],
        icon: "ti ti-bell"
      }, {
        default: _withCtx(() => [_createElementVNode("div", { class: "_gaps_m" }, [
          _createVNode(MkFeatureBanner, {
            icon: "/client-assets/bell_3d.png",
            color: "#ffff00"
          }, {
            default: _withCtx(() => [_createVNode(_component_SearchText, null, {
              default: _withCtx(() => [_createTextVNode(
                _toDisplayString(_unref(i18n).ts._settings.notificationsBanner),
                1
                /* TEXT */
              )]),
              _: 1
            })]),
            _: 1
          }),
          _createVNode(FormSection, { first: "" }, {
            label: _withCtx(() => [_createTextVNode(
              _toDisplayString(_unref(i18n).ts.notificationRecieveConfig),
              1
              /* TEXT */
            )]),
            default: _withCtx(() => [_createElementVNode("div", { class: "_gaps_s" }, [(_openBlock(true), _createElementBlock(
              _Fragment,
              null,
              _renderList(_unref(configurableNotificationTypes), (type) => {
                return _openBlock(), _createBlock(
                  MkFolder,
                  { key: type },
                  {
                    label: _withCtx(() => [_createTextVNode(
                      _toDisplayString(_unref(i18n).ts._notification._types[type]),
                      1
                      /* TEXT */
                    )]),
                    suffix: _withCtx(() => [_createTextVNode(
                      _toDisplayString(_unref($i).notificationRecieveConfig[type]?.type === "never" ? _unref(i18n).ts.none : _unref($i).notificationRecieveConfig[type]?.type === "following" ? _unref(i18n).ts.following : _unref($i).notificationRecieveConfig[type]?.type === "follower" ? _unref(i18n).ts.followers : _unref($i).notificationRecieveConfig[type]?.type === "mutualFollow" ? _unref(i18n).ts.mutualFollow : _unref($i).notificationRecieveConfig[type]?.type === "followingOrFollower" ? _unref(i18n).ts.followingOrFollower : _unref($i).notificationRecieveConfig[type]?.type === "list" ? _unref(i18n).ts.userList : _unref(i18n).ts.all),
                      1
                      /* TEXT */
                    )]),
                    default: _withCtx(() => [_createVNode(XNotificationConfig, {
                      userLists: _unref(userLists),
                      value: _unref($i).notificationRecieveConfig[type] ?? { type: "all" },
                      configurableTypes: _unref(onlyOnOrOffNotificationTypes).includes(type) ? ["all", "never"] : undefined,
                      onUpdate: (res) => updateReceiveConfig(type, res)
                    }, null, 8, [
                      "userLists",
                      "value",
                      "configurableTypes",
                      "onUpdate"
                    ])]),
                    _: 2
                  },
                  1024
                  /* DYNAMIC_SLOTS */
                );
              }),
              128
              /* KEYED_FRAGMENT */
            ))])]),
            _: 1
          }),
          _createVNode(FormSection, null, {
            default: _withCtx(() => [_createElementVNode("div", { class: "_gaps_m" }, [_createVNode(FormLink, { to: "/settings/sounds" }, {
              default: _withCtx(() => [_createTextVNode(
                _toDisplayString(_unref(i18n).ts.notificationSoundSettings),
                1
                /* TEXT */
              )]),
              _: 1
            })])]),
            _: 1
          }),
          _createVNode(FormSection, null, {
            default: _withCtx(() => [_createElementVNode("div", { class: "_gaps_s" }, [
              _createVNode(MkButton, { onClick: readAllNotifications }, {
                default: _withCtx(() => [_createTextVNode(
                  _toDisplayString(_unref(i18n).ts.markAsReadAllNotifications),
                  1
                  /* TEXT */
                )]),
                _: 1
              }),
              _createVNode(MkButton, { onClick: testNotification }, {
                default: _withCtx(() => [_createTextVNode(
                  _toDisplayString(_unref(i18n).ts._notification.sendTestNotification),
                  1
                  /* TEXT */
                )]),
                _: 1
              }),
              _createVNode(MkButton, { onClick: flushNotification }, {
                default: _withCtx(() => [_createTextVNode(
                  _toDisplayString(_unref(i18n).ts._notification.flushNotification),
                  1
                  /* TEXT */
                )]),
                _: 1
              })
            ])]),
            _: 1
          }),
          _createVNode(FormSection, null, {
            label: _withCtx(() => [_createTextVNode(
              _toDisplayString(_unref(i18n).ts.pushNotification),
              1
              /* TEXT */
            )]),
            default: _withCtx(() => [_createElementVNode("div", { class: "_gaps_m" }, [_createVNode(
              MkPushNotificationAllowButton,
              {
                ref_key: "allowButton",
                ref: allowButton
              },
              null,
              512
              /* NEED_PATCH */
            ), _createVNode(MkSwitch, {
              disabled: !pushRegistrationInServer.value,
              modelValue: sendReadMessage.value,
              "onUpdate:modelValue": onChangeSendReadMessage
            }, {
              label: _withCtx(() => [_createTextVNode(
                _toDisplayString(_unref(i18n).ts.sendPushNotificationReadMessage),
                1
                /* TEXT */
              )]),
              caption: _withCtx(() => [_createTextVNode(
                _toDisplayString(_unref(i18n).ts.sendPushNotificationReadMessageCaption),
                1
                /* TEXT */
              )]),
              _: 1
            }, 8, ["disabled", "modelValue"])])]),
            _: 1
          })
        ])]),
        _: 1
      }, 8, ["label", "keywords"]);
    };
  }
};
