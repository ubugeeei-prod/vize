import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-settings-question" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-users" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-planet" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("br")
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("br")
const _hoisted_6 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-mail" })
const _hoisted_7 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-adjustments-alt" })
const _hoisted_8 = /*#__PURE__*/ _createElementVNode("b", null, "Log IP:")
const _hoisted_9 = /*#__PURE__*/ _createElementVNode("b", null, "FTT:")
const _hoisted_10 = /*#__PURE__*/ _createElementVNode("b", null, "RBT:")
const _hoisted_11 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-check" })
import { computed, ref } from 'vue'
import * as Misskey from 'misskey-js'
import MkButton from '@/components/MkButton.vue'
import MkInput from '@/components/MkInput.vue'
import * as os from '@/os.js'
import { misskeyApi } from '@/utility/misskey-api.js'
import { i18n } from '@/i18n.js'
import MkFolder from '@/components/MkFolder.vue'
import MkRadios from '@/components/MkRadios.vue'
import MkSwitch from '@/components/MkSwitch.vue'
import MkInfo from '@/components/MkInfo.vue'
import MkLink from '@/components/MkLink.vue'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkServerSetupWizard',
  props: {
    token: { type: String, required: false }
  },
  emits: ["finished"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
const q_name = ref('');
const q_use = ref<'single' | 'group' | 'open'>('single');
const q_scale = ref<'small' | 'medium' | 'large'>('small');
const q_federation = ref<'yes' | 'no'>('no');
const q_remoteContentsCleaning = ref(true);
const q_adminName = ref('');
const q_adminEmail = ref('');
const serverSettings = computed<Misskey.entities.AdminUpdateMetaRequest>(() => {
	let enableReactionsBuffering;
	if (q_use.value === 'single') {
		enableReactionsBuffering = false;
	} else {
		enableReactionsBuffering = q_scale.value !== 'small';
	}

	return {
		singleUserMode: q_use.value === 'single',
		disableRegistration: q_use.value !== 'open',
		emailRequiredForSignup: q_use.value === 'open',
		enableIpLogging: q_use.value === 'open',
		federation: q_federation.value === 'yes' ? 'all' : 'none',
		enableRemoteNotesCleaning: q_remoteContentsCleaning.value,
		enableFanoutTimeline: true,
		enableFanoutTimelineDbFallback: q_use.value === 'single',
		enableReactionsBuffering,
		clientOptions: {
			entrancePageStyle: q_use.value === 'open' ? 'classic' : 'simple',
		},
	};
});
const defaultPolicies = computed<Partial<Misskey.entities.RolePolicies>>(() => {
	let driveCapacityMb: Misskey.entities.RolePolicies['driveCapacityMb'] | undefined;
	if (q_use.value === 'single') {
		driveCapacityMb = 8192;
	} else if (q_use.value === 'group') {
		driveCapacityMb = 1000;
	} else if (q_use.value === 'open') {
		driveCapacityMb = 100;
	}

	let rateLimitFactor: Misskey.entities.RolePolicies['rateLimitFactor'] | undefined;
	if (q_use.value === 'single') {
		rateLimitFactor = 0.3;
	} else if (q_use.value === 'group') {
		rateLimitFactor = 0.7;
	} else if (q_use.value === 'open') {
		if (q_scale.value === 'small') {
			rateLimitFactor = 1;
		} else if (q_scale.value === 'medium') {
			rateLimitFactor = 1.25;
		} else if (q_scale.value === 'large') {
			rateLimitFactor = 1.5;
		}
	}

	let userListLimit: Misskey.entities.RolePolicies['userListLimit'] | undefined;
	if (q_use.value === 'single') {
		userListLimit = 100;
	} else if (q_use.value === 'group') {
		userListLimit = 5;
	} else if (q_use.value === 'open') {
		userListLimit = 3;
	}

	let antennaLimit: Misskey.entities.RolePolicies['antennaLimit'] | undefined;
	if (q_use.value === 'single') {
		antennaLimit = 100;
	} else if (q_use.value === 'group') {
		antennaLimit = 5;
	} else if (q_use.value === 'open') {
		antennaLimit = 0;
	}

	let webhookLimit: Misskey.entities.RolePolicies['webhookLimit'] | undefined;
	if (q_use.value === 'single') {
		webhookLimit = 100;
	} else if (q_use.value === 'group') {
		webhookLimit = 0;
	} else if (q_use.value === 'open') {
		webhookLimit = 0;
	}

	let canImportFollowing: Misskey.entities.RolePolicies['canImportFollowing'];
	if (q_use.value === 'single') {
		canImportFollowing = true;
	} else {
		canImportFollowing = false;
	}

	let canImportMuting: Misskey.entities.RolePolicies['canImportMuting'];
	if (q_use.value === 'single') {
		canImportMuting = true;
	} else {
		canImportMuting = false;
	}

	let canImportBlocking: Misskey.entities.RolePolicies['canImportBlocking'];
	if (q_use.value === 'single') {
		canImportBlocking = true;
	} else {
		canImportBlocking = false;
	}

	let canImportUserLists: Misskey.entities.RolePolicies['canImportUserLists'];
	if (q_use.value === 'single') {
		canImportUserLists = true;
	} else {
		canImportUserLists = false;
	}

	let canImportAntennas: Misskey.entities.RolePolicies['canImportAntennas'];
	if (q_use.value === 'single') {
		canImportAntennas = true;
	} else {
		canImportAntennas = false;
	}

	return {
		rateLimitFactor,
		driveCapacityMb,
		userListLimit,
		antennaLimit,
		webhookLimit,
		canImportFollowing,
		canImportMuting,
		canImportBlocking,
		canImportUserLists,
		canImportAntennas,
	};
});
function applySettings() {
	const _close = os.waiting();
	Promise.all([
		misskeyApi('admin/update-meta', {
			...serverSettings.value,
			name: q_name.value === '' ? undefined : q_name.value,
			maintainerName: q_adminName.value === '' ? undefined : q_adminName.value,
			maintainerEmail: q_adminEmail.value === '' ? undefined : q_adminEmail.value,
		}, props.token),
		misskeyApi('admin/roles/update-default-policies', {
			// @ts-expect-error バックエンド側の型
			policies: defaultPolicies.value,
		}, props.token),
	]).then(() => {
		emit('finished');
	}).catch((err) => {
		os.alert({
			type: 'error',
			title: err.code,
			text: err.message,
		});
	}).finally(() => {
		_close();
	});
}

return (_ctx: any,_cache: any) => {
  return (_openBlock(), _createElementBlock("div", { class: "_gaps_m" }, [ _createVNode(MkInput, {
        "data-cy-server-name": "",
        modelValue: q_name.value,
        "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((q_name).value = $event))
      }, {
        label: _withCtx(() => [
          _createTextVNode(_toDisplayString(_unref(i18n).ts.instanceName), 1 /* TEXT */)
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["modelValue"]), _createVNode(MkFolder, { defaultOpen: true }, {
        label: _withCtx(() => [
          _createTextVNode(_toDisplayString(_unref(i18n).ts._serverSetupWizard.howWillYouUseMisskey), 1 /* TEXT */)
        ]),
        icon: _withCtx(() => [
          _hoisted_1
        ]),
        default: _withCtx(() => [
          _createElementVNode("div", { class: "_gaps_s" }, [
            _createVNode(MkRadios, {
              options: [
  					{ value: 'single', label: _unref(i18n).ts._serverSetupWizard._use.single, icon: 'ti ti-user', caption: _unref(i18n).ts._serverSetupWizard._use.single_description },
  					{ value: 'group', label: _unref(i18n).ts._serverSetupWizard._use.group, icon: 'ti ti-lock', caption: _unref(i18n).ts._serverSetupWizard._use.group_description },
  					{ value: 'open', label: _unref(i18n).ts._serverSetupWizard._use.open, icon: 'ti ti-world', caption: _unref(i18n).ts._serverSetupWizard._use.open_description },
  				],
              vertical: "",
              modelValue: q_use.value,
              "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event: any) => ((q_use).value = $event))
            }, null, 8 /* PROPS */, ["options", "modelValue"]),
            (q_use.value === 'single')
              ? (_openBlock(), _createBlock(MkInfo, { key: 0 }, {
                default: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._serverSetupWizard._use.single_youCanCreateMultipleAccounts), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }))
              : _createCommentVNode("v-if", true),
            (q_use.value === 'open')
              ? (_openBlock(), _createBlock(MkInfo, {
                key: 0,
                warn: ""
              }, {
                default: _withCtx(() => [
                  _createElementVNode("b", null, _toDisplayString(_unref(i18n).ts.advice) + ":", 1 /* TEXT */),
                  _createTextVNode(" "),
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._serverSetupWizard.openServerAdvice), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }))
              : _createCommentVNode("v-if", true),
            (q_use.value === 'open')
              ? (_openBlock(), _createBlock(MkInfo, {
                key: 0,
                warn: ""
              }, {
                default: _withCtx(() => [
                  _createElementVNode("b", null, _toDisplayString(_unref(i18n).ts.advice) + ":", 1 /* TEXT */),
                  _createTextVNode(" "),
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._serverSetupWizard.openServerAntiSpamAdvice), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }))
              : _createCommentVNode("v-if", true)
          ])
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["defaultOpen"]), (q_use.value !== 'single') ? (_openBlock(), _createBlock(MkFolder, {
          key: 0,
          defaultOpen: true
        }, {
          label: _withCtx(() => [
            _createTextVNode(_toDisplayString(_unref(i18n).ts._serverSetupWizard.howManyUsersDoYouExpect), 1 /* TEXT */)
          ]),
          icon: _withCtx(() => [
            _hoisted_2
          ]),
          default: _withCtx(() => [
            _createElementVNode("div", { class: "_gaps_s" }, [
              _createVNode(MkRadios, {
                options: [
  					{ value: 'small', label: _unref(i18n).ts._serverSetupWizard._scale.small, icon: 'ti ti-user' },
  					{ value: 'medium', label: _unref(i18n).ts._serverSetupWizard._scale.medium, icon: 'ti ti-users' },
  					{ value: 'large', label: _unref(i18n).ts._serverSetupWizard._scale.large, icon: 'ti ti-users-group' },
  				],
                vertical: "",
                modelValue: q_scale.value,
                "onUpdate:modelValue": _cache[2] || (_cache[2] = ($event: any) => ((q_scale).value = $event))
              }, null, 8 /* PROPS */, ["options", "modelValue"]),
              (q_scale.value === 'large')
                ? (_openBlock(), _createBlock(MkInfo, { key: 0 }, {
                  default: _withCtx(() => [
                    _createElementVNode("b", null, _toDisplayString(_unref(i18n).ts.advice) + ":", 1 /* TEXT */),
                    _createTextVNode(" "),
                    _createTextVNode(_toDisplayString(_unref(i18n).ts._serverSetupWizard.largeScaleServerAdvice), 1 /* TEXT */)
                  ]),
                  _: 1 /* STABLE */
                }))
                : _createCommentVNode("v-if", true)
            ])
          ]),
          _: 1 /* STABLE */
        }, 8 /* PROPS */, ["defaultOpen"])) : _createCommentVNode("v-if", true), _createVNode(MkFolder, { defaultOpen: true }, {
        label: _withCtx(() => [
          _createTextVNode(_toDisplayString(_unref(i18n).ts._serverSetupWizard.doYouConnectToFediverse), 1 /* TEXT */)
        ]),
        icon: _withCtx(() => [
          _hoisted_3
        ]),
        default: _withCtx(() => [
          _createElementVNode("div", { class: "_gaps_s" }, [
            _createElementVNode("div", null, [
              _createTextVNode(_toDisplayString(_unref(i18n).ts._serverSetupWizard.doYouConnectToFediverse_description1), 1 /* TEXT */),
              _hoisted_4,
              _createTextVNode(_toDisplayString(_unref(i18n).ts._serverSetupWizard.doYouConnectToFediverse_description2), 1 /* TEXT */),
              _hoisted_5,
              _createVNode(MkLink, {
                target: "_blank",
                url: "https://wikipedia.org/wiki/Fediverse"
              }, {
                default: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts.learnMore), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              })
            ]),
            _createVNode(MkRadios, {
              options: [
  					{ value: 'yes', label: _unref(i18n).ts.yes },
  					{ value: 'no', label: _unref(i18n).ts.no },
  				],
              vertical: "",
              modelValue: q_federation.value,
              "onUpdate:modelValue": _cache[3] || (_cache[3] = ($event: any) => ((q_federation).value = $event))
            }, null, 8 /* PROPS */, ["options", "modelValue"]),
            (q_federation.value === 'yes')
              ? (_openBlock(), _createBlock(MkInfo, { key: 0 }, {
                default: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._serverSetupWizard.youCanConfigureMoreFederationSettingsLater), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }))
              : _createCommentVNode("v-if", true),
            (q_federation.value === 'yes')
              ? (_openBlock(), _createBlock(MkSwitch, {
                key: 0,
                modelValue: q_remoteContentsCleaning.value,
                "onUpdate:modelValue": _cache[4] || (_cache[4] = ($event: any) => ((q_remoteContentsCleaning).value = $event))
              }, {
                label: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._serverSetupWizard.remoteContentsCleaning), 1 /* TEXT */)
                ]),
                caption: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._serverSetupWizard.remoteContentsCleaning_description), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["modelValue"]))
              : _createCommentVNode("v-if", true)
          ])
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["defaultOpen"]), (q_use.value === 'open' || q_federation.value === 'yes') ? (_openBlock(), _createBlock(MkFolder, {
          key: 0,
          defaultOpen: true
        }, {
          label: _withCtx(() => [
            _createTextVNode(_toDisplayString(_unref(i18n).ts._serverSetupWizard.adminInfo), 1 /* TEXT */)
          ]),
          icon: _withCtx(() => [
            _hoisted_6
          ]),
          default: _withCtx(() => [
            _createElementVNode("div", { class: "_gaps_s" }, [
              _createElementVNode("div", null, _toDisplayString(_unref(i18n).ts._serverSetupWizard.adminInfo_description), 1 /* TEXT */),
              _createVNode(MkInfo, { warn: "" }, {
                default: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._serverSetupWizard.adminInfo_mustBeFilled), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }),
              _createVNode(MkInput, {
                modelValue: q_adminName.value,
                "onUpdate:modelValue": _cache[5] || (_cache[5] = ($event: any) => ((q_adminName).value = $event))
              }, {
                label: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts.maintainerName), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["modelValue"]),
              _createVNode(MkInput, {
                type: "email",
                modelValue: q_adminEmail.value,
                "onUpdate:modelValue": _cache[6] || (_cache[6] = ($event: any) => ((q_adminEmail).value = $event))
              }, {
                label: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts.maintainerEmail), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["modelValue"])
            ])
          ]),
          _: 1 /* STABLE */
        }, 8 /* PROPS */, ["defaultOpen"])) : _createCommentVNode("v-if", true), _createVNode(MkFolder, {
        defaultOpen: true,
        maxHeight: 300
      }, {
        label: _withCtx(() => [
          _createTextVNode(_toDisplayString(_unref(i18n).ts._serverSetupWizard.followingSettingsAreRecommended), 1 /* TEXT */)
        ]),
        icon: _withCtx(() => [
          _hoisted_7
        ]),
        footer: _withCtx(() => [
          _createVNode(MkButton, {
            gradate: "",
            large: "",
            rounded: "",
            "data-cy-server-setup-wizard-apply": "",
            style: "margin: 0 auto;",
            onClick: applySettings
          }, {
            default: _withCtx(() => [
              _hoisted_11,
              _createTextVNode(" "),
              _createTextVNode(_toDisplayString(_unref(i18n).ts._serverSetupWizard.applyTheseSettings), 1 /* TEXT */)
            ]),
            _: 1 /* STABLE */
          })
        ]),
        default: _withCtx(() => [
          _createElementVNode("div", { class: "_gaps_s" }, [
            _createElementVNode("div", null, [
              _createElementVNode("div", null, [
                _createElementVNode("b", null, _toDisplayString(_unref(i18n).ts._serverSettings.singleUserMode) + ":", 1 /* TEXT */)
              ]),
              _createElementVNode("div", null, _toDisplayString(serverSettings.value.singleUserMode ? _unref(i18n).ts.yes : _unref(i18n).ts.no), 1 /* TEXT */)
            ]),
            _createElementVNode("div", null, [
              _createElementVNode("div", null, [
                _createElementVNode("b", null, _toDisplayString(_unref(i18n).ts._serverSettings.openRegistration) + ":", 1 /* TEXT */)
              ]),
              _createElementVNode("div", null, _toDisplayString(!serverSettings.value.disableRegistration ? _unref(i18n).ts.yes : _unref(i18n).ts.no), 1 /* TEXT */)
            ]),
            _createElementVNode("div", null, [
              _createElementVNode("div", null, [
                _createElementVNode("b", null, _toDisplayString(_unref(i18n).ts.emailRequiredForSignup) + ":", 1 /* TEXT */)
              ]),
              _createElementVNode("div", null, _toDisplayString(serverSettings.value.emailRequiredForSignup ? _unref(i18n).ts.yes : _unref(i18n).ts.no), 1 /* TEXT */)
            ]),
            _createElementVNode("div", null, [
              _createElementVNode("div", null, [
                _hoisted_8
              ]),
              _createElementVNode("div", null, _toDisplayString(serverSettings.value.enableIpLogging ? _unref(i18n).ts.yes : _unref(i18n).ts.no), 1 /* TEXT */)
            ]),
            _createElementVNode("div", null, [
              _createElementVNode("div", null, [
                _createElementVNode("b", null, _toDisplayString(_unref(i18n).ts.federation) + ":", 1 /* TEXT */)
              ]),
              _createElementVNode("div", null, _toDisplayString(serverSettings.value.federation === 'none' ? _unref(i18n).ts.no : _unref(i18n).ts.all), 1 /* TEXT */)
            ]),
            _createElementVNode("div", null, [
              _createElementVNode("div", null, [
                _createElementVNode("b", null, _toDisplayString(_unref(i18n).ts._serverSettings.remoteNotesCleaning) + ":", 1 /* TEXT */)
              ]),
              _createElementVNode("div", null, _toDisplayString(serverSettings.value.enableRemoteNotesCleaning ? _unref(i18n).ts.yes : _unref(i18n).ts.no), 1 /* TEXT */)
            ]),
            _createElementVNode("div", null, [
              _createElementVNode("div", null, [
                _hoisted_9
              ]),
              _createElementVNode("div", null, _toDisplayString(serverSettings.value.enableFanoutTimeline ? _unref(i18n).ts.yes : _unref(i18n).ts.no), 1 /* TEXT */)
            ]),
            _createElementVNode("div", null, [
              _createElementVNode("div", null, [
                _createElementVNode("b", null, "FTT/" + _toDisplayString(_unref(i18n).ts._serverSettings.fanoutTimelineDbFallback) + ":", 1 /* TEXT */)
              ]),
              _createElementVNode("div", null, _toDisplayString(serverSettings.value.enableFanoutTimelineDbFallback ? _unref(i18n).ts.yes : _unref(i18n).ts.no), 1 /* TEXT */)
            ]),
            _createElementVNode("div", null, [
              _createElementVNode("div", null, [
                _hoisted_10
              ]),
              _createElementVNode("div", null, _toDisplayString(serverSettings.value.enableReactionsBuffering ? _unref(i18n).ts.yes : _unref(i18n).ts.no), 1 /* TEXT */)
            ]),
            _createElementVNode("div", null, [
              _createElementVNode("div", null, [
                _createElementVNode("b", null, _toDisplayString(_unref(i18n).ts._serverSettings.entrancePageStyle) + ":", 1 /* TEXT */)
              ]),
              _createElementVNode("div", null, _toDisplayString(serverSettings.value.clientOptions?.entrancePageStyle), 1 /* TEXT */)
            ]),
            _createElementVNode("div", null, [
              _createElementVNode("div", null, [
                _createElementVNode("b", null, _toDisplayString(_unref(i18n).ts._role.baseRole) + "/" + _toDisplayString(_unref(i18n).ts._role._options.rateLimitFactor) + ":", 1 /* TEXT */)
              ]),
              _createElementVNode("div", null, _toDisplayString(defaultPolicies.value.rateLimitFactor), 1 /* TEXT */)
            ]),
            _createElementVNode("div", null, [
              _createElementVNode("div", null, [
                _createElementVNode("b", null, _toDisplayString(_unref(i18n).ts._role.baseRole) + "/" + _toDisplayString(_unref(i18n).ts._role._options.driveCapacity) + ":", 1 /* TEXT */)
              ]),
              _createElementVNode("div", null, _toDisplayString(defaultPolicies.value.driveCapacityMb) + " MB", 1 /* TEXT */)
            ]),
            _createElementVNode("div", null, [
              _createElementVNode("div", null, [
                _createElementVNode("b", null, _toDisplayString(_unref(i18n).ts._role.baseRole) + "/" + _toDisplayString(_unref(i18n).ts._role._options.userListMax) + ":", 1 /* TEXT */)
              ]),
              _createElementVNode("div", null, _toDisplayString(defaultPolicies.value.userListLimit), 1 /* TEXT */)
            ]),
            _createElementVNode("div", null, [
              _createElementVNode("div", null, [
                _createElementVNode("b", null, _toDisplayString(_unref(i18n).ts._role.baseRole) + "/" + _toDisplayString(_unref(i18n).ts._role._options.antennaMax) + ":", 1 /* TEXT */)
              ]),
              _createElementVNode("div", null, _toDisplayString(defaultPolicies.value.antennaLimit), 1 /* TEXT */)
            ]),
            _createElementVNode("div", null, [
              _createElementVNode("div", null, [
                _createElementVNode("b", null, _toDisplayString(_unref(i18n).ts._role.baseRole) + "/" + _toDisplayString(_unref(i18n).ts._role._options.webhookMax) + ":", 1 /* TEXT */)
              ]),
              _createElementVNode("div", null, _toDisplayString(defaultPolicies.value.webhookLimit), 1 /* TEXT */)
            ]),
            _createElementVNode("div", null, [
              _createElementVNode("div", null, [
                _createElementVNode("b", null, _toDisplayString(_unref(i18n).ts._role.baseRole) + "/" + _toDisplayString(_unref(i18n).ts._role._options.canImportFollowing) + ":", 1 /* TEXT */)
              ]),
              _createElementVNode("div", null, _toDisplayString(defaultPolicies.value.canImportFollowing ? _unref(i18n).ts.yes : _unref(i18n).ts.no), 1 /* TEXT */)
            ]),
            _createElementVNode("div", null, [
              _createElementVNode("div", null, [
                _createElementVNode("b", null, _toDisplayString(_unref(i18n).ts._role.baseRole) + "/" + _toDisplayString(_unref(i18n).ts._role._options.canImportMuting) + ":", 1 /* TEXT */)
              ]),
              _createElementVNode("div", null, _toDisplayString(defaultPolicies.value.canImportMuting ? _unref(i18n).ts.yes : _unref(i18n).ts.no), 1 /* TEXT */)
            ]),
            _createElementVNode("div", null, [
              _createElementVNode("div", null, [
                _createElementVNode("b", null, _toDisplayString(_unref(i18n).ts._role.baseRole) + "/" + _toDisplayString(_unref(i18n).ts._role._options.canImportBlocking) + ":", 1 /* TEXT */)
              ]),
              _createElementVNode("div", null, _toDisplayString(defaultPolicies.value.canImportBlocking ? _unref(i18n).ts.yes : _unref(i18n).ts.no), 1 /* TEXT */)
            ]),
            _createElementVNode("div", null, [
              _createElementVNode("div", null, [
                _createElementVNode("b", null, _toDisplayString(_unref(i18n).ts._role.baseRole) + "/" + _toDisplayString(_unref(i18n).ts._role._options.canImportUserLists) + ":", 1 /* TEXT */)
              ]),
              _createElementVNode("div", null, _toDisplayString(defaultPolicies.value.canImportUserLists ? _unref(i18n).ts.yes : _unref(i18n).ts.no), 1 /* TEXT */)
            ]),
            _createElementVNode("div", null, [
              _createElementVNode("div", null, [
                _createElementVNode("b", null, _toDisplayString(_unref(i18n).ts._role.baseRole) + "/" + _toDisplayString(_unref(i18n).ts._role._options.canImportAntennas) + ":", 1 /* TEXT */)
              ]),
              _createElementVNode("div", null, _toDisplayString(defaultPolicies.value.canImportAntennas ? _unref(i18n).ts.yes : _unref(i18n).ts.no), 1 /* TEXT */)
            ])
          ])
        ]),
        _: 1 /* STABLE */
      }, 8 /* PROPS */, ["defaultOpen", "maxHeight"]) ]))
}
}

})
