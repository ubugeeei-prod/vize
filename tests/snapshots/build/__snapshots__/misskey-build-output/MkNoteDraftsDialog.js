import { defineComponent as _defineComponent } from 'vue'
import { Fragment as _Fragment, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, resolveDirective as _resolveDirective, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue"


const _hoisted_1 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-arrow-back-up" })
const _hoisted_2 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-arrow-back-up" })
const _hoisted_3 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-quote" })
const _hoisted_4 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-quote" })
const _hoisted_5 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-device-tv" })
const _hoisted_6 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-rocket-off" })
const _hoisted_7 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-calendar-x" })
const _hoisted_8 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-corner-up-left" })
const _hoisted_9 = /*#__PURE__*/ _createElementVNode("i", { class: "ti ti-trash" })
import { ref, shallowRef, markRaw } from 'vue'
import * as Misskey from 'misskey-js'
import MkButton from '@/components/MkButton.vue'
import MkPagination from '@/components/MkPagination.vue'
import MkModalWindow from '@/components/MkModalWindow.vue'
import { getNoteSummary } from '@/utility/get-note-summary.js'
import { i18n } from '@/i18n.js'
import * as os from '@/os.js'
import { $i } from '@/i.js'
import { misskeyApi } from '@/utility/misskey-api'
import { Paginator } from '@/utility/paginator.js'
import MkTabs from '@/components/MkTabs.vue'
import MkInfo from '@/components/MkInfo.vue'

export default /*@__PURE__*/_defineComponent({
  __name: 'MkNoteDraftsDialog',
  props: {
    scheduled: { type: Boolean, required: false }
  },
  emits: ["restore", "cancel", "closed"],
  setup(__props: any, { emit: __emit }) {

const emit = __emit
const props = __props
const tab = ref<'drafts' | 'scheduled'>(props.scheduled ? 'scheduled' : 'drafts');
const draftsPaginator = markRaw(new Paginator('notes/drafts/list', {
	limit: 10,
	params: {
		scheduled: false,
	},
}));
const scheduledPaginator = markRaw(new Paginator('notes/drafts/list', {
	limit: 10,
	params: {
		scheduled: true,
	},
}));
const currentDraftsCount = ref(0);
misskeyApi('notes/drafts/count').then((count) => {
	currentDraftsCount.value = count;
});
const dialogEl = shallowRef<InstanceType<typeof MkModalWindow>>();
function cancel() {
	emit('cancel');
	dialogEl.value?.close();
}
function restoreDraft(draft: Misskey.entities.NoteDraft) {
	emit('restore', draft);
	dialogEl.value?.close();
}
async function deleteDraft(draft: Misskey.entities.NoteDraft) {
	const { canceled } = await os.confirm({
		type: 'warning',
		text: i18n.ts._drafts.deleteAreYouSure,
	});
	if (canceled) return;
	os.apiWithDialog('notes/drafts/delete', { draftId: draft.id }).then(() => {
		draftsPaginator.reload();
	});
}
async function cancelSchedule(draft: Misskey.entities.NoteDraft) {
	os.apiWithDialog('notes/drafts/update', {
		draftId: draft.id,
		isActuallyScheduled: false,
		scheduledAt: null,
	}).then(() => {
		scheduledPaginator.reload();
	});
}

return (_ctx: any,_cache: any) => {
  const _component_MkResult = _resolveComponent("MkResult")
  const _component_MkTime = _resolveComponent("MkTime")
  const _component_I18n = _resolveComponent("I18n")
  const _component_Mfm = _resolveComponent("Mfm")
  const _component_MkAcct = _resolveComponent("MkAcct")
  const _component_MkStickyContainer = _resolveComponent("MkStickyContainer")
  const _directive_panel = _resolveDirective("panel")
  const _directive_tooltip = _resolveDirective("tooltip")

  return (_openBlock(), _createBlock(MkModalWindow, {
      ref_key: "dialogEl", ref: dialogEl,
      width: 600,
      height: 650,
      withOkButton: false,
      onClick: _cache[0] || (_cache[0] = ($event: any) => (cancel())),
      onClose: _cache[1] || (_cache[1] = ($event: any) => (cancel())),
      onClosed: _cache[2] || (_cache[2] = ($event: any) => (emit('closed'))),
      onEsc: _cache[3] || (_cache[3] = ($event: any) => (cancel()))
    }, {
      header: _withCtx(() => [
        _createTextVNode(_toDisplayString(_unref(i18n).ts.draftsAndScheduledNotes) + " (" + _toDisplayString(currentDraftsCount.value) + "/" + _toDisplayString(_unref($i)?.policies.noteDraftLimit) + ")\n\t", 1 /* TEXT */)
      ]),
      default: _withCtx(() => [
        _createVNode(_component_MkStickyContainer, null, {
          header: _withCtx(() => [
            _createVNode(MkTabs, {
              centered: "",
              class: _normalizeClass(_ctx.$style.tabs),
              tabs: [
  					{
  						key: 'drafts',
  						title: _unref(i18n).ts.drafts,
  						icon: 'ti ti-pencil-question',
  					},
  					{
  						key: 'scheduled',
  						title: _unref(i18n).ts.scheduled,
  						icon: 'ti ti-calendar-clock',
  					},
  				],
              tab: tab.value,
              "onUpdate:tab": _cache[4] || (_cache[4] = ($event: any) => ((tab).value = $event))
            }, null, 8 /* PROPS */, ["tabs", "tab"])
          ]),
          default: _withCtx(() => [
            _createElementVNode("div", { class: "_spacer" }, [
              _createVNode(MkPagination, {
                key: tab.value,
                paginator: tab.value === 'scheduled' ? _unref(scheduledPaginator) : _unref(draftsPaginator),
                withControl: ""
              }, {
                empty: _withCtx(() => [
                  _createVNode(_component_MkResult, {
                    type: "empty",
                    text: _unref(i18n).ts._drafts.noDrafts
                  }, null, 8 /* PROPS */, ["text"])
                ]),
                default: _withCtx(({ items }) => [
                  _createElementVNode("div", { class: "_gaps_s" }, [
                    (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(items, (draft) => {
                      return (_openBlock(), _createElementBlock("div", {
                        key: draft.id,
                        class: _normalizeClass([_ctx.$style.draft])
                      }, [
                        _createElementVNode("div", {
                          class: _normalizeClass(["_gaps_s", _ctx.$style.draftBody])
                        }, [
                          (draft.scheduledAt != null && draft.isActuallyScheduled)
                            ? (_openBlock(), _createBlock(MkInfo, { key: 0 }, {
                              default: _withCtx(() => [
                                _createVNode(_component_I18n, {
                                  src: _unref(i18n).ts.scheduledToPostOnX,
                                  tag: "span"
                                }, {
                                  x: _withCtx(() => [
                                    _createVNode(_component_MkTime, {
                                      time: draft.scheduledAt,
                                      mode: 'detail',
                                      style: "font-weight: bold;"
                                    }, null, 8 /* PROPS */, ["time", "mode"])
                                  ]),
                                  _: 2 /* DYNAMIC */
                                }, 8 /* PROPS */, ["src"])
                              ]),
                              _: 2 /* DYNAMIC */
                            }))
                            : _createCommentVNode("v-if", true),
                          _createElementVNode("div", {
                            class: _normalizeClass(_ctx.$style.draftInfo)
                          }, [
                            _createElementVNode("div", {
                              class: _normalizeClass(_ctx.$style.draftMeta)
                            }, [
                              (draft.reply)
                                ? (_openBlock(), _createElementBlock("div", {
                                  key: 0,
                                  class: "_nowrap"
                                }, [
                                  _hoisted_1,
                                  _createTextVNode(),
                                  _createVNode(_component_I18n, {
                                    src: _unref(i18n).ts._drafts.replyTo,
                                    tag: "span"
                                  }, {
                                    user: _withCtx(() => [
                                      (draft.reply.user.name != null)
                                        ? (_openBlock(), _createBlock(_component_Mfm, {
                                          key: 0,
                                          text: draft.reply.user.name,
                                          plain: true,
                                          nowrap: true
                                        }, null, 8 /* PROPS */, ["text", "plain", "nowrap"]))
                                        : (_openBlock(), _createBlock(_component_MkAcct, {
                                          key: 1,
                                          user: draft.reply.user
                                        }, null, 8 /* PROPS */, ["user"]))
                                    ]),
                                    _: 2 /* DYNAMIC */
                                  }, 8 /* PROPS */, ["src"])
                                ]))
                                : (draft.replyId)
                                  ? (_openBlock(), _createElementBlock("div", {
                                    key: 1,
                                    class: "_nowrap"
                                  }, [
                                    _hoisted_2,
                                    _createTextVNode(),
                                    _createVNode(_component_I18n, {
                                      src: _unref(i18n).ts._drafts.replyTo,
                                      tag: "span"
                                    }, {
                                      user: _withCtx(() => [
                                        _createTextVNode(_toDisplayString(_unref(i18n).ts.deletedNote), 1 /* TEXT */)
                                      ]),
                                      _: 2 /* DYNAMIC */
                                    }, 8 /* PROPS */, ["src"])
                                  ]))
                                : _createCommentVNode("v-if", true),
                              (draft.renote && draft.text != null)
                                ? (_openBlock(), _createElementBlock("div", {
                                  key: 0,
                                  class: "_nowrap"
                                }, [
                                  _hoisted_3,
                                  _createTextVNode(),
                                  _createVNode(_component_I18n, {
                                    src: _unref(i18n).ts._drafts.quoteOf,
                                    tag: "span"
                                  }, {
                                    user: _withCtx(() => [
                                      (draft.renote.user.name != null)
                                        ? (_openBlock(), _createBlock(_component_Mfm, {
                                          key: 0,
                                          text: draft.renote.user.name,
                                          plain: true,
                                          nowrap: true
                                        }, null, 8 /* PROPS */, ["text", "plain", "nowrap"]))
                                        : (_openBlock(), _createBlock(_component_MkAcct, {
                                          key: 1,
                                          user: draft.renote.user
                                        }, null, 8 /* PROPS */, ["user"]))
                                    ]),
                                    _: 2 /* DYNAMIC */
                                  }, 8 /* PROPS */, ["src"])
                                ]))
                                : (draft.renoteId)
                                  ? (_openBlock(), _createElementBlock("div", {
                                    key: 1,
                                    class: "_nowrap"
                                  }, [
                                    _hoisted_4,
                                    _createTextVNode(),
                                    _createVNode(_component_I18n, {
                                      src: _unref(i18n).ts._drafts.quoteOf,
                                      tag: "span"
                                    }, {
                                      user: _withCtx(() => [
                                        _createTextVNode(_toDisplayString(_unref(i18n).ts.deletedNote), 1 /* TEXT */)
                                      ]),
                                      _: 2 /* DYNAMIC */
                                    }, 8 /* PROPS */, ["src"])
                                  ]))
                                : _createCommentVNode("v-if", true),
                              (draft.channel)
                                ? (_openBlock(), _createElementBlock("div", {
                                  key: 0,
                                  class: "_nowrap"
                                }, [
                                  _hoisted_5,
                                  _createTextVNode(),
                                  _toDisplayString(_unref(i18n).tsx._drafts.postTo({ channel: draft.channel.name }))
                                ]))
                                : _createCommentVNode("v-if", true)
                            ])
                          ]),
                          _createElementVNode("div", {
                            class: _normalizeClass(_ctx.$style.draftContent)
                          }, [
                            _createVNode(_component_Mfm, {
                              text: _unref(getNoteSummary)(draft, { showRenote: false, showReply: false }),
                              plain: true,
                              author: draft.user
                            }, null, 8 /* PROPS */, ["text", "plain", "author"])
                          ]),
                          _createElementVNode("div", {
                            class: _normalizeClass(_ctx.$style.draftFooter)
                          }, [
                            _createElementVNode("div", {
                              class: _normalizeClass(_ctx.$style.draftVisibility)
                            }, [
                              _createElementVNode("span", { title: _unref(i18n).ts._visibility[draft.visibility] }, [
                                (draft.visibility === 'public')
                                  ? (_openBlock(), _createElementBlock("i", {
                                    key: 0,
                                    class: "ti ti-world"
                                  }))
                                  : (draft.visibility === 'home')
                                    ? (_openBlock(), _createElementBlock("i", {
                                      key: 1,
                                      class: "ti ti-home"
                                    }))
                                  : (draft.visibility === 'followers')
                                    ? (_openBlock(), _createElementBlock("i", {
                                      key: 2,
                                      class: "ti ti-lock"
                                    }))
                                  : (draft.visibility === 'specified')
                                    ? (_openBlock(), _createElementBlock("i", {
                                      key: 3,
                                      class: "ti ti-mail"
                                    }))
                                  : _createCommentVNode("v-if", true)
                              ], 8 /* PROPS */, ["title"]),
                              (draft.localOnly)
                                ? (_openBlock(), _createElementBlock("span", {
                                  key: 0,
                                  title: _unref(i18n).ts._visibility['disableFederation']
                                }, [
                                  _hoisted_6
                                ]))
                                : _createCommentVNode("v-if", true)
                            ]),
                            _createVNode(_component_MkTime, {
                              time: draft.createdAt,
                              class: _normalizeClass(_ctx.$style.draftCreatedAt),
                              mode: "detail",
                              colored: ""
                            }, null, 8 /* PROPS */, ["time"])
                          ])
                        ]),
                        _createElementVNode("div", {
                          class: _normalizeClass(["_buttons", _ctx.$style.draftActions])
                        }, [
                          (draft.scheduledAt != null && draft.isActuallyScheduled)
                            ? (_openBlock(), _createBlock(MkButton, {
                              key: 0,
                              small: "",
                              onClick: _cache[5] || (_cache[5] = ($event: any) => (cancelSchedule(draft)))
                            }, {
                              default: _withCtx(() => [
                                _hoisted_7,
                                _createTextVNode(" "),
                                _createTextVNode(_toDisplayString(_unref(i18n).ts._drafts.cancelSchedule), 1 /* TEXT */)
                              ]),
                              _: 2 /* DYNAMIC */
                            }))
                            : (_openBlock(), _createBlock(MkButton, {
                              key: 1,
                              small: "",
                              onClick: _cache[6] || (_cache[6] = ($event: any) => (restoreDraft(draft)))
                            }, {
                              default: _withCtx(() => [
                                _hoisted_8,
                                _createTextVNode(" "),
                                _createTextVNode(_toDisplayString(_unref(i18n).ts._drafts.restore), 1 /* TEXT */)
                              ]),
                              _: 2 /* DYNAMIC */
                            })),
                          _createVNode(MkButton, {
                            danger: "",
                            small: "",
                            iconOnly: true,
                            style: "margin-left: auto;",
                            onClick: _cache[7] || (_cache[7] = ($event: any) => (deleteDraft(draft)))
                          }, {
                            default: _withCtx(() => [
                              _hoisted_9
                            ]),
                            _: 2 /* DYNAMIC */
                          }, 8 /* PROPS */, ["iconOnly"])
                        ])
                      ]))
                    }), 128 /* KEYED_FRAGMENT */))
                  ])
                ]),
                _: 1 /* STABLE */
              }, 8 /* PROPS */, ["paginator"])
            ])
          ]),
          _: 1 /* STABLE */
        })
      ]),
      _: 1 /* STABLE */
    }, 8 /* PROPS */, ["width", "height", "withOkButton"]))
}
}

})
