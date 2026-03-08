import { defineComponent as _defineComponent } from 'vue'
import { openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, resolveDirective as _resolveDirective, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, normalizeStyle as _normalizeStyle, withCtx as _withCtx, unref as _unref, withModifiers as _withModifiers } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-sparkles" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-map-pin" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-cake" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-list" })
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-check" })
const _hoisted_6 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-plus" })
const _hoisted_7 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-trash" })
const _hoisted_8 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-arrows-sort" })
const _hoisted_9 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-menu" })
const _hoisted_10 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-x" })
const _hoisted_11 = /*#__PURE__*/ _createElementVNode("hr")
const _hoisted_12 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-qrcode" })
import { computed, reactive, ref, watch } from 'vue'
import * as Misskey from 'misskey-js'
import MkButton from '@/components/MkButton.vue'
import MkInput from '@/components/MkInput.vue'
import MkSwitch from '@/components/MkSwitch.vue'
import MkSelect from '@/components/MkSelect.vue'
import FormSplit from '@/components/form/split.vue'
import MkFolder from '@/components/MkFolder.vue'
import FormSlot from '@/components/form/slot.vue'
import FormLink from '@/components/form/link.vue'
import MkDraggable from '@/components/MkDraggable.vue'
import { chooseDriveFile } from '@/utility/drive.js'
import * as os from '@/os.js'
import { i18n } from '@/i18n.js'
import { ensureSignin } from '@/i.js'
import { langmap } from '@/utility/langmap.js'
import { definePage } from '@/page.js'
import { claimAchievement } from '@/utility/achievements.js'
import { store } from '@/store.js'
import MkInfo from '@/components/MkInfo.vue'
import MkTextarea from '@/components/MkTextarea.vue'
import { genId } from '@/utility/id.js'

export default /*@__PURE__*/_defineComponent({
  __name: 'profile',
  setup(__props) {

const $i = ensureSignin();
const reactionAcceptance = store.model('reactionAcceptance');
function assertVaildLang(lang: string | null): lang is keyof typeof langmap {
	return lang != null && lang in langmap;
}
const profile = reactive({
	name: $i.name,
	description: $i.description,
	followedMessage: $i.followedMessage,
	location: $i.location,
	birthday: $i.birthday,
	lang: assertVaildLang($i.lang) ? $i.lang : null,
	isBot: $i.isBot ?? false,
	isCat: $i.isCat ?? false,
});
watch(() => profile, () => {
	save();
}, {
	deep: true,
});
const fields = ref($i.fields.map(field => ({ id: genId(), name: field.name, value: field.value })) ?? []);
const fieldEditMode = ref(false);
function addField() {
	fields.value.push({
		id: genId(),
		name: '',
		value: '',
	});
}
while (fields.value.length < 4) {
	addField();
}
function deleteField(itemId: string) {
	fields.value = fields.value.filter(f => f.id !== itemId);
}
function saveFields() {
	os.apiWithDialog('i/update', {
		fields: fields.value.filter(field => field.name !== '' && field.value !== '').map(field => ({ name: field.name, value: field.value })),
	});
}
function save() {
	os.apiWithDialog('i/update', {
		// 空文字列をnullにしたいので??は使うな
		// eslint-disable-next-line @typescript-eslint/prefer-nullish-coalescing
		name: profile.name || null,
		// eslint-disable-next-line @typescript-eslint/prefer-nullish-coalescing
		description: profile.description || null,
		// eslint-disable-next-line @typescript-eslint/prefer-nullish-coalescing
		followedMessage: profile.followedMessage || null,
		// eslint-disable-next-line @typescript-eslint/prefer-nullish-coalescing
		location: profile.location || null,
		// eslint-disable-next-line @typescript-eslint/prefer-nullish-coalescing
		birthday: profile.birthday || null,
		// eslint-disable-next-line @typescript-eslint/prefer-nullish-coalescing
		lang: profile.lang || null,
		isBot: !!profile.isBot,
		isCat: !!profile.isCat,
	}, undefined, {
		'0b3f9f6a-2f4d-4b1f-9fb4-49d3a2fd7191': {
			title: i18n.ts.yourNameContainsProhibitedWords,
			text: i18n.ts.yourNameContainsProhibitedWordsDescription,
		},
	});
	claimAchievement('profileFilled');
	if (profile.name === 'syuilo' || profile.name === 'しゅいろ') {
		claimAchievement('setNameToSyuilo');
	}
	if (profile.isCat) {
		claimAchievement('markedAsCat');
	}
}
function changeAvatar(ev: PointerEvent) {
	async function done(driveFile: Misskey.entities.DriveFile) {
		const i = await os.apiWithDialog('i/update', {
			avatarId: driveFile.id,
		});
		$i.avatarId = i.avatarId;
		$i.avatarUrl = i.avatarUrl;
		claimAchievement('profileFilled');
	}
	os.popupMenu([{
		text: i18n.ts.avatar,
		type: 'label',
	}, {
		text: i18n.ts.upload,
		icon: 'ti ti-upload',
		action: async () => {
			const files = await os.chooseFileFromPc({ multiple: false });
			const file = files[0];
			let originalOrCropped = file;
			const { canceled } = await os.confirm({
				type: 'question',
				text: i18n.ts.cropImageAsk,
				okText: i18n.ts.cropYes,
				cancelText: i18n.ts.cropNo,
			});
			if (!canceled) {
				originalOrCropped = await os.cropImageFile(file, {
					aspectRatio: 1,
				});
			}
			const driveFile = (await os.launchUploader([originalOrCropped], { multiple: false }))[0];
			done(driveFile);
		},
	}, {
		text: i18n.ts.fromDrive,
		icon: 'ti ti-cloud',
		action: () => {
			chooseDriveFile({ multiple: false }).then(files => {
				done(files[0]);
			});
		},
	}], ev.currentTarget ?? ev.target);
}
function changeBanner(ev: PointerEvent) {
	async function done(driveFile: Misskey.entities.DriveFile) {
		const i = await os.apiWithDialog('i/update', {
			bannerId: driveFile.id,
		});
		$i.bannerId = i.bannerId;
		$i.bannerUrl = i.bannerUrl;
	}
	os.popupMenu([{
		text: i18n.ts.banner,
		type: 'label',
	}, {
		text: i18n.ts.upload,
		icon: 'ti ti-upload',
		action: async () => {
			const files = await os.chooseFileFromPc({ multiple: false });
			const file = files[0];
			let originalOrCropped = file;
			const { canceled } = await os.confirm({
				type: 'question',
				text: i18n.ts.cropImageAsk,
				okText: i18n.ts.cropYes,
				cancelText: i18n.ts.cropNo,
			});
			if (!canceled) {
				originalOrCropped = await os.cropImageFile(file, {
					aspectRatio: 2,
				});
			}
			const driveFile = (await os.launchUploader([originalOrCropped], { multiple: false }))[0];
			done(driveFile);
		},
	}, {
		text: i18n.ts.fromDrive,
		icon: 'ti ti-cloud',
		action: () => {
			chooseDriveFile({ multiple: false }).then(files => {
				done(files[0]);
			});
		},
	}], ev.currentTarget ?? ev.target);
}
const headerActions = computed(() => []);
const headerTabs = computed(() => []);
definePage(() => ({
	title: i18n.ts.profile,
	icon: 'ti ti-user',
}));

return (_ctx: any,_cache: any) => {
  const _component_SearchLabel = _resolveComponent("SearchLabel")
  const _component_SearchMarker = _resolveComponent("SearchMarker")
  const _component_MkAvatar = _resolveComponent("MkAvatar")
  const _component_SearchText = _resolveComponent("SearchText")
  const _directive_panel = _resolveDirective("panel")

  return (_openBlock(), _createBlock(_component_SearchMarker, {
      path: "/settings/profile",
      label: _unref(i18n).ts.profile,
      keywords: ['profile'],
      icon: "ti ti-user"
    }, {
      default: _withCtx(() => [
        _createElementVNode("div", { class: "_gaps_m" }, [
          _createElementVNode("div", { class: "_panel" }, [
            _createElementVNode("div", {
              class: _normalizeClass(_ctx.$style.banner),
              style: _normalizeStyle({ backgroundImage: _unref($i).bannerUrl ? `url(${ _unref($i).bannerUrl })` : '' })
            }, [
              _createElementVNode("div", {
                class: _normalizeClass(_ctx.$style.bannerEdit)
              }, [
                _createVNode(_component_SearchMarker, { keywords: ['banner', 'change'] }, {
                  default: _withCtx(() => [
                    _createVNode(MkButton, {
                      primary: "",
                      rounded: "",
                      onClick: changeBanner
                    }, {
                      default: _withCtx(() => [
                        _createVNode(_component_SearchLabel, null, {
                          default: _withCtx(() => [
                            _createTextVNode(_toDisplayString(_unref(i18n).ts._profile.changeBanner), 1 /* TEXT */)
                          ]),
                          _: 1 /* STABLE */
                        })
                      ]),
                      _: 1 /* STABLE */
                    })
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["keywords"])
              ])
            ], 4 /* STYLE */),
            _createElementVNode("div", {
              class: _normalizeClass(_ctx.$style.avatarContainer)
            }, [
              _createVNode(_component_MkAvatar, {
                class: _normalizeClass(_ctx.$style.avatar),
                user: _unref($i),
                forceShowDecoration: "",
                onClick: changeAvatar
              }, null, 8 /* PROPS */, ["user"]),
              _createElementVNode("div", { class: "_buttonsCenter" }, [
                _createVNode(_component_SearchMarker, { keywords: ['avatar', 'icon', 'change'] }, {
                  default: _withCtx(() => [
                    _createVNode(MkButton, {
                      primary: "",
                      rounded: "",
                      onClick: changeAvatar
                    }, {
                      default: _withCtx(() => [
                        _createVNode(_component_SearchLabel, null, {
                          default: _withCtx(() => [
                            _createTextVNode(_toDisplayString(_unref(i18n).ts._profile.changeAvatar), 1 /* TEXT */)
                          ]),
                          _: 1 /* STABLE */
                        })
                      ]),
                      _: 1 /* STABLE */
                    })
                  ]),
                  _: 1 /* STABLE */
                }, 8 /* PROPS */, ["keywords"]),
                _createVNode(MkButton, {
                  primary: "",
                  rounded: "",
                  link: "",
                  to: "/settings/avatar-decoration"
                }, {
                  default: _withCtx(() => [
                    _createTextVNode(_toDisplayString(_unref(i18n).ts.decorate), 1 /* TEXT */),
                    _createTextVNode(" "),
                    _hoisted_1
                  ]),
                  _: 1 /* STABLE */
                })
              ])
            ])
          ]),
          _createVNode(_component_SearchMarker, { keywords: ['name'] }, {
            default: _withCtx(() => [
              _createVNode(MkInput, {
                max: 30,
                manualSave: "",
                mfmAutocomplete: ['emoji'],
                modelValue: profile.name,
                "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event: any) => ((profile.name) = $event))
              }, {
                label: _withCtx(() => [
                  _createVNode(_component_SearchLabel, null, {
                    default: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_unref(i18n).ts._profile.name), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  })
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["max", "mfmAutocomplete", "modelValue"])
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["keywords"]),
          _createVNode(_component_SearchMarker, { keywords: ['description', 'bio'] }, {
            default: _withCtx(() => [
              _createVNode(MkTextarea, {
                max: 500,
                tall: "",
                manualSave: "",
                mfmAutocomplete: "",
                mfmPreview: true,
                modelValue: profile.description,
                "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event: any) => ((profile.description) = $event))
              }, {
                label: _withCtx(() => [
                  _createVNode(_component_SearchLabel, null, {
                    default: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_unref(i18n).ts._profile.description), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  })
                ]),
                caption: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._profile.youCanIncludeHashtags), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["max", "mfmPreview", "modelValue"])
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["keywords"]),
          _createVNode(_component_SearchMarker, { keywords: ['location', 'locale'] }, {
            default: _withCtx(() => [
              _createVNode(MkInput, {
                manualSave: "",
                modelValue: profile.location,
                "onUpdate:modelValue": _cache[2] || (_cache[2] = ($event: any) => ((profile.location) = $event))
              }, {
                label: _withCtx(() => [
                  _createVNode(_component_SearchLabel, null, {
                    default: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_unref(i18n).ts.location), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  })
                ]),
                prefix: _withCtx(() => [
                  _hoisted_2
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["modelValue"])
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["keywords"]),
          _createVNode(_component_SearchMarker, { keywords: ['birthday', 'birthdate', 'age'] }, {
            default: _withCtx(() => [
              _createVNode(MkInput, {
                type: "date",
                manualSave: "",
                modelValue: profile.birthday,
                "onUpdate:modelValue": _cache[3] || (_cache[3] = ($event: any) => ((profile.birthday) = $event))
              }, {
                label: _withCtx(() => [
                  _createVNode(_component_SearchLabel, null, {
                    default: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_unref(i18n).ts.birthday), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  })
                ]),
                prefix: _withCtx(() => [
                  _hoisted_3
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["modelValue"])
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["keywords"]),
          _createVNode(_component_SearchMarker, { keywords: ['language', 'locale'] }, {
            default: _withCtx(() => [
              _createVNode(MkSelect, {
                items: Object.entries(_unref(langmap)).map(([code, def]) => ({
  	label: def.nativeName,
  	value: code
  })),
                modelValue: profile.lang,
                "onUpdate:modelValue": _cache[4] || (_cache[4] = ($event: any) => ((profile.lang) = $event))
              }, {
                label: _withCtx(() => [
                  _createVNode(_component_SearchLabel, null, {
                    default: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_unref(i18n).ts.language), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  })
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["items", "modelValue"])
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["keywords"]),
          _createVNode(_component_SearchMarker, { keywords: ['metadata'] }, {
            default: _withCtx(() => [
              _createVNode(FormSlot, null, {
                caption: _withCtx(() => [
                  _createTextVNode(_toDisplayString(_unref(i18n).ts._profile.metadataDescription), 1 /* TEXT */)
                ]),
                default: _withCtx(() => [
                  _createVNode(MkFolder, null, {
                    icon: _withCtx(() => [
                      _hoisted_4
                    ]),
                    label: _withCtx(() => [
                      _createVNode(_component_SearchLabel, null, {
                        default: _withCtx(() => [
                          _createTextVNode(_toDisplayString(_unref(i18n).ts._profile.metadataEdit), 1 /* TEXT */)
                        ]),
                        _: 1 /* STABLE */
                      })
                    ]),
                    footer: _withCtx(() => [
                      _createElementVNode("div", { class: "_buttons" }, [
                        _createVNode(MkButton, {
                          primary: "",
                          onClick: saveFields
                        }, {
                          default: _withCtx(() => [
                            _hoisted_5,
                            _createTextVNode(" "),
                            _createTextVNode(_toDisplayString(_unref(i18n).ts.save), 1 /* TEXT */)
                          ]),
                          _: 1 /* STABLE */
                        }),
                        _createVNode(MkButton, {
                          disabled: fields.value.length >= 16,
                          onClick: addField
                        }, {
                          default: _withCtx(() => [
                            _hoisted_6,
                            _createTextVNode(" "),
                            _createTextVNode(_toDisplayString(_unref(i18n).ts.add), 1 /* TEXT */)
                          ]),
                          _: 1 /* STABLE */
                        }, 8 /* PROPS */, ["disabled"]),
                        (!fieldEditMode.value)
                          ? (_openBlock(), _createBlock(MkButton, {
                            key: 0,
                            disabled: fields.value.length <= 1,
                            danger: "",
                            onClick: _cache[5] || (_cache[5] = ($event: any) => (fieldEditMode.value = !fieldEditMode.value))
                          }, {
                            default: _withCtx(() => [
                              _hoisted_7,
                              _createTextVNode(" "),
                              _createTextVNode(_toDisplayString(_unref(i18n).ts.delete), 1 /* TEXT */)
                            ]),
                            _: 1 /* STABLE */
                          }, 8 /* PROPS */, ["disabled"]))
                          : (_openBlock(), _createBlock(MkButton, {
                            key: 1,
                            onClick: _cache[6] || (_cache[6] = ($event: any) => (fieldEditMode.value = !fieldEditMode.value))
                          }, {
                            default: _withCtx(() => [
                              _hoisted_8,
                              _createTextVNode(" "),
                              _createTextVNode(_toDisplayString(_unref(i18n).ts.rearrange), 1 /* TEXT */)
                            ]),
                            _: 1 /* STABLE */
                          }))
                      ])
                    ]),
                    default: _withCtx(() => [
                      _createElementVNode("div", {
                        class: _normalizeClass(["_gaps_s", _ctx.$style.metadataRoot])
                      }, [
                        _createVNode(MkInfo, null, {
                          default: _withCtx(() => [
                            _createTextVNode(_toDisplayString(_unref(i18n).ts._profile.verifiedLinkDescription), 1 /* TEXT */)
                          ]),
                          _: 1 /* STABLE */
                        }),
                        _createVNode(MkDraggable, {
                          direction: "vertical",
                          withGaps: "",
                          manualDragStart: "",
                          modelValue: fields.value,
                          "onUpdate:modelValue": _cache[7] || (_cache[7] = ($event: any) => ((fields).value = $event))
                        }, {
                          default: _withCtx(({ item, dragStart }) => [
                            _createElementVNode("div", {
                              class: _normalizeClass(_ctx.$style.fieldDragItem)
                            }, [
                              (!fieldEditMode.value)
                                ? (_openBlock(), _createElementBlock("button", {
                                  key: 0,
                                  class: _normalizeClass(["_button", _ctx.$style.dragItemHandle]),
                                  tabindex: "-1",
                                  draggable: true,
                                  onDragstart: _cache[8] || (_cache[8] = _withModifiers((...args) => (_ctx.dragStart && _ctx.dragStart(...args)), ["stop"]))
                                }, [
                                  _hoisted_9
                                ]))
                                : _createCommentVNode("v-if", true),
                              (fieldEditMode.value)
                                ? (_openBlock(), _createElementBlock("button", {
                                  key: 0,
                                  disabled: fields.value.length <= 1,
                                  class: _normalizeClass(["_button", _ctx.$style.dragItemRemove]),
                                  onClick: _cache[9] || (_cache[9] = ($event: any) => (deleteField(_ctx.item.id)))
                                }, [
                                  _hoisted_10
                                ]))
                                : _createCommentVNode("v-if", true),
                              _createElementVNode("div", {
                                class: _normalizeClass(_ctx.$style.dragItemForm)
                              }, [
                                _createVNode(FormSplit, { minWidth: 200 }, {
                                  default: _withCtx(() => [
                                    _createVNode(MkInput, {
                                      small: "",
                                      placeholder: _unref(i18n).ts._profile.metadataLabel,
                                      modelValue: item.name,
                                      "onUpdate:modelValue": _cache[10] || (_cache[10] = ($event: any) => ((_ctx.item.name) = $event))
                                    }, null, 8 /* PROPS */, ["placeholder", "modelValue"]),
                                    _createVNode(MkInput, {
                                      small: "",
                                      placeholder: _unref(i18n).ts._profile.metadataContent,
                                      modelValue: item.value,
                                      "onUpdate:modelValue": _cache[11] || (_cache[11] = ($event: any) => ((_ctx.item.value) = $event))
                                    }, null, 8 /* PROPS */, ["placeholder", "modelValue"])
                                  ]),
                                  _: 1 /* STABLE */
                                }, 8 /* PROPS */, ["minWidth"])
                              ])
                            ])
                          ]),
                          _: 1 /* STABLE */
                        }, 8 /* PROPS */, ["modelValue"])
                      ])
                    ]),
                    _: 1 /* STABLE */
                  })
                ]),
                _: 1 /* STABLE */
              })
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["keywords"]),
          _createVNode(_component_SearchMarker, { keywords: ['follow', 'message'] }, {
            default: _withCtx(() => [
              _createVNode(MkInput, {
                max: 200,
                manualSave: "",
                mfmPreview: false,
                modelValue: profile.followedMessage,
                "onUpdate:modelValue": _cache[12] || (_cache[12] = ($event: any) => ((profile.followedMessage) = $event))
              }, {
                label: _withCtx(() => [
                  _createVNode(_component_SearchLabel, null, {
                    default: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_unref(i18n).ts._profile.followedMessage), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  })
                ]),
                caption: _withCtx(() => [
                  _createElementVNode("div", null, [
                    _createVNode(_component_SearchText, null, {
                      default: _withCtx(() => [
                        _createTextVNode(_toDisplayString(_unref(i18n).ts._profile.followedMessageDescription), 1 /* TEXT */)
                      ]),
                      _: 1 /* STABLE */
                    })
                  ]),
                  _createElementVNode("div", null, _toDisplayString(_unref(i18n).ts._profile.followedMessageDescriptionForLockedAccount), 1 /* TEXT */)
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["max", "mfmPreview", "modelValue"])
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["keywords"]),
          _createVNode(_component_SearchMarker, { keywords: ['reaction'] }, {
            default: _withCtx(() => [
              _createVNode(MkSelect, {
                items: [
  					{ label: _unref(i18n).ts.all, value: null },
  					{ label: _unref(i18n).ts.likeOnlyForRemote, value: 'likeOnlyForRemote' },
  					{ label: _unref(i18n).ts.nonSensitiveOnly, value: 'nonSensitiveOnly' },
  					{ label: _unref(i18n).ts.nonSensitiveOnlyForLocalLikeOnlyForRemote, value: 'nonSensitiveOnlyForLocalLikeOnlyForRemote' },
  					{ label: _unref(i18n).ts.likeOnly, value: 'likeOnly' },
  				],
                modelValue: _unref(reactionAcceptance),
                "onUpdate:modelValue": _cache[13] || (_cache[13] = ($event: any) => ((reactionAcceptance).value = $event))
              }, {
                label: _withCtx(() => [
                  _createVNode(_component_SearchLabel, null, {
                    default: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_unref(i18n).ts.reactionAcceptance), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  })
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["items", "modelValue"])
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["keywords"]),
          _createVNode(_component_SearchMarker, null, {
            default: _withCtx(() => [
              _createVNode(MkFolder, null, {
                label: _withCtx(() => [
                  _createVNode(_component_SearchLabel, null, {
                    default: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_unref(i18n).ts.advancedSettings), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  })
                ]),
                default: _withCtx(() => [
                  _createElementVNode("div", { class: "_gaps_m" }, [
                    _createVNode(_component_SearchMarker, { keywords: ['cat'] }, {
                      default: _withCtx(() => [
                        _createVNode(MkSwitch, {
                          modelValue: profile.isCat,
                          "onUpdate:modelValue": _cache[14] || (_cache[14] = ($event: any) => ((profile.isCat) = $event))
                        }, {
                          label: _withCtx(() => [
                            _createVNode(_component_SearchLabel, null, {
                              default: _withCtx(() => [
                                _createTextVNode(_toDisplayString(_unref(i18n).ts.flagAsCat), 1 /* TEXT */)
                              ]),
                              _: 1 /* STABLE */
                            })
                          ]),
                          caption: _withCtx(() => [
                            _createTextVNode(_toDisplayString(_unref(i18n).ts.flagAsCatDescription), 1 /* TEXT */)
                          ]),
                          _: 1 /* STABLE */
                        }, 8 /* PROPS */, ["modelValue"])
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["keywords"]),
                    _createVNode(_component_SearchMarker, { keywords: ['bot'] }, {
                      default: _withCtx(() => [
                        _createVNode(MkSwitch, {
                          modelValue: profile.isBot,
                          "onUpdate:modelValue": _cache[15] || (_cache[15] = ($event: any) => ((profile.isBot) = $event))
                        }, {
                          label: _withCtx(() => [
                            _createVNode(_component_SearchLabel, null, {
                              default: _withCtx(() => [
                                _createTextVNode(_toDisplayString(_unref(i18n).ts.flagAsBot), 1 /* TEXT */)
                              ]),
                              _: 1 /* STABLE */
                            })
                          ]),
                          caption: _withCtx(() => [
                            _createTextVNode(_toDisplayString(_unref(i18n).ts.flagAsBotDescription), 1 /* TEXT */)
                          ]),
                          _: 1 /* STABLE */
                        }, 8 /* PROPS */, ["modelValue"])
                      ]),
                      _: 1 /* STABLE */
                    }, 8 /* PROPS */, ["keywords"])
                  ])
                ]),
                _: 1 /* STABLE */
              })
            ]),
            _: 1 /* STABLE */
          }),
          _hoisted_11,
          _createVNode(_component_SearchMarker, { keywords: ['qrcode'] }, {
            default: _withCtx(() => [
              _createVNode(FormLink, { to: "/qr" }, {
                icon: _withCtx(() => [
                  _hoisted_12
                ]),
                default: _withCtx(() => [
                  _createVNode(_component_SearchLabel, null, {
                    default: _withCtx(() => [
                      _createTextVNode(_toDisplayString(_unref(i18n).ts.qr), 1 /* TEXT */)
                    ]),
                    _: 1 /* STABLE */
                  })
                ]),
                _: 1 /* STABLE */
              })
            ]),
            _: 1 /* STABLE */
          }, 8 /* PROPS */, ["keywords"])
        ])
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["label", "keywords"]))
}
}

})
