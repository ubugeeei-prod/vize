import { renderList as _renderList, Fragment as _Fragment, openBlock as _openBlock, createElementBlock as _createElementBlock, createBlock as _createBlock, createCommentVNode as _createCommentVNode, toDisplayString as _toDisplayString, createTextVNode as _createTextVNode, withCtx as _withCtx, createVNode as _createVNode, withKeys as _withKeys, normalizeClass as _normalizeClass, createElementVNode as _createElementVNode, createSlots as _createSlots, TransitionGroup as _TransitionGroup } from "vue"

export function render(_ctx: any,_cache: any,$props: any,$setup: any,$data: any,$options: any) {
  return (_openBlock(), _createBlock(_TransitionGroup, { name: "confirmation-slide" }, {
    default: _withCtx(() => [
      (_openBlock(true), _createElementBlock(_Fragment, null, _renderList($setup.chunks, (chunk) => {
        return (_openBlock(), _createElementBlock(_Fragment, {
          key: chunk.item.toolCall.confirmation.requestId
        }, [
          (
					chunk.type === 'floating' &&
					chunk.item.toolCall.confirmation.inputType === 'questions' &&
					chunk.item.toolCall.confirmation.questions
				)
            ? (_openBlock(), _createBlock($setup["InstanceAiQuestions"], {
                key: 'q-' + chunk.item.toolCall.confirmation.requestId,
                questions: chunk.item.toolCall.confirmation.questions!,
                "intro-message": chunk.item.toolCall.confirmation.introMessage,
                onSubmit: (answers) => $setup.handleQuestionsSubmit(chunk.item.toolCall.confirmation, answers)
              }, null, 8 /* PROPS */, ["questions", "intro-message", "onSubmit"]))
            : (chunk.type === 'standalone')
              ? (_openBlock(), _createElementBlock(_Fragment, { key: 1 }, [
                  (chunk.item.toolCall.confirmation.setupRequests?.length)
                    ? (_openBlock(), _createBlock($setup["InstanceAiWorkflowSetup"], {
                        key: 'setup-' + chunk.item.toolCall.confirmation.requestId,
                        "request-id": chunk.item.toolCall.confirmation.requestId,
                        "setup-requests": chunk.item.toolCall.confirmation.setupRequests!,
                        "project-id": chunk.item.toolCall.confirmation.projectId ?? $setup.thread.projectId,
                        "credential-flow": chunk.item.toolCall.confirmation.credentialFlow,
                        "workflow-id": chunk.item.toolCall.confirmation.workflowId
                      }, null, 8 /* PROPS */, ["request-id", "setup-requests", "project-id", "credential-flow", "workflow-id"]))
                    : (chunk.item.toolCall.confirmation.credentialRequests?.length)
                      ? (_openBlock(), _createBlock($setup["InstanceAiCredentialSetup"], {
                          key: 'cred-' + chunk.item.toolCall.confirmation.requestId,
                          "request-id": chunk.item.toolCall.confirmation.requestId,
                          "credential-requests": chunk.item.toolCall.confirmation.credentialRequests!,
                          message: chunk.item.toolCall.confirmation.message,
                          "project-id": chunk.item.toolCall.confirmation.projectId ?? $setup.thread.projectId,
                          "credential-flow": chunk.item.toolCall.confirmation.credentialFlow,
                          "require-user-selection": chunk.item.toolCall.confirmation.requireUserSelection
                        }, null, 8 /* PROPS */, ["request-id", "credential-requests", "message", "project-id", "credential-flow", "require-user-selection"]))
                      : (chunk.item.toolCall.confirmation.inputType === 'text')
                        ? (_openBlock(), _createElementBlock("div", {
                            key: 'text-' + chunk.item.toolCall.confirmation.requestId
                          }, [
                            _createVNode($setup["N8nCard"], {
                              class: _normalizeClass(_ctx.$style.textCard)
                            }, {
                              default: _withCtx(() => [
                                _createVNode($setup["N8nText"], { tag: "div" }, {
                                  default: _withCtx(() => [
                                    _createTextVNode(_toDisplayString(chunk.item.toolCall.confirmation!.message), 1 /* TEXT */)
                                  ]),
                                  _: 2 /* DYNAMIC */
                                }, 1024 /* DYNAMIC_SLOTS */),
                                _createElementVNode("div", {
                                  class: _normalizeClass(_ctx.$style.textInputRow)
                                }, [
                                  _createVNode($setup["N8nInput"], {
                                    modelValue: $setup.textInputValues[chunk.item.toolCall.confirmation!.requestId],
                                    "onUpdate:modelValue": ($event: any) => (($setup.textInputValues[chunk.item.toolCall.confirmation!.requestId]) = $event),
                                    type: "text",
                                    size: "small",
                                    placeholder: $setup.i18n.baseText('instanceAi.askUser.placeholder'),
                                    onKeydown: _withKeys(($event: any) => ($setup.handleTextSubmit(chunk.item.toolCall.confirmation)), ["enter"])
                                  }, null, 8 /* PROPS */, ["modelValue", "onUpdate:modelValue", "placeholder", "onKeydown"]),
                                  (!($setup.textInputValues[chunk.item.toolCall.confirmation.requestId] ?? '').trim())
                                    ? (_openBlock(), _createBlock($setup["N8nButton"], {
                                        key: 0,
                                        size: "medium",
                                        variant: "outline",
                                        onClick: ($event: any) => ($setup.handleTextSkip(chunk.item.toolCall.confirmation))
                                      }, {
                                        default: _withCtx(() => [
                                          _createTextVNode(_toDisplayString($setup.i18n.baseText('instanceAi.askUser.skip')), 1 /* TEXT */)
                                        ]),
                                        _: 1 /* STABLE */
                                      }, 8 /* PROPS */, ["onClick"]))
                                    : _createCommentVNode("v-if", true),
                                  _createVNode($setup["N8nButton"], {
                                    size: "medium",
                                    variant: "solid",
                                    disabled: 
									!($setup.textInputValues[chunk.item.toolCall.confirmation.requestId] ?? '').trim()
								,
                                    onClick: ($event: any) => ($setup.handleTextSubmit(chunk.item.toolCall.confirmation))
                                  }, {
                                    default: _withCtx(() => [
                                      _createTextVNode(_toDisplayString($setup.i18n.baseText('instanceAi.askUser.submit')), 1 /* TEXT */)
                                    ]),
                                    _: 1 /* STABLE */
                                  }, 8 /* PROPS */, ["disabled", "onClick"])
                                ], 2 /* CLASS */)
                              ]),
                              _: 2 /* DYNAMIC */
                            }, 1032 /* PROPS, DYNAMIC_SLOTS */, ["class"])
                          ]))
                        : (chunk.item.toolCall.confirmation.inputType === 'continue')
                          ? (_openBlock(), _createElementBlock("div", {
                              key: 'continue-' + chunk.item.toolCall.confirmation.requestId
                            }, [
                              _createVNode($setup["N8nCard"], {
                                class: _normalizeClass(_ctx.$style.textCard)
                              }, {
                                default: _withCtx(() => [
                                  _createVNode($setup["N8nText"], { tag: "div" }, {
                                    default: _withCtx(() => [
                                      _createTextVNode(_toDisplayString(chunk.item.toolCall.confirmation!.message), 1 /* TEXT */)
                                    ]),
                                    _: 2 /* DYNAMIC */
                                  }, 1024 /* DYNAMIC_SLOTS */),
                                  _createElementVNode("div", {
                                    class: _normalizeClass(_ctx.$style.continueRow)
                                  }, [
                                    _createVNode($setup["N8nButton"], {
                                      "data-test-id": "instance-ai-panel-continue",
                                      size: "medium",
                                      variant: "solid",
                                      onClick: ($event: any) => ($setup.handleContinue(chunk.item.toolCall.confirmation))
                                    }, {
                                      default: _withCtx(() => [
                                        _createTextVNode(_toDisplayString($setup.i18n.baseText('instanceAi.confirmation.continue')), 1 /* TEXT */)
                                      ]),
                                      _: 1 /* STABLE */
                                    }, 8 /* PROPS */, ["onClick"])
                                  ], 2 /* CLASS */)
                                ]),
                                _: 2 /* DYNAMIC */
                              }, 1032 /* PROPS, DYNAMIC_SLOTS */, ["class"])
                            ]))
                          : (chunk.item.toolCall.confirmation.testListener)
                            ? (_openBlock(), _createElementBlock("div", {
                                key: 'test-listener-' + chunk.item.toolCall.confirmation.requestId,
                                "data-test-id": "instance-ai-test-listener"
                              }, [
                                _createVNode($setup["N8nCard"], {
                                  class: _normalizeClass(_ctx.$style.textCard)
                                }, {
                                  default: _withCtx(() => [
                                    _createVNode($setup["N8nText"], { tag: "div" }, {
                                      default: _withCtx(() => [
                                        _createTextVNode(_toDisplayString(chunk.item.toolCall.confirmation.message), 1 /* TEXT */)
                                      ]),
                                      _: 2 /* DYNAMIC */
                                    }, 1024 /* DYNAMIC_SLOTS */),
                                    (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(chunk.item.toolCall.confirmation.testListener.triggers, (trigger) => {
                                      return (_openBlock(), _createElementBlock("div", {
                                        key: trigger.nodeName,
                                        class: _normalizeClass(_ctx.$style.testListenerUrl)
                                      }, [
                                        _createVNode($setup["N8nText"], {
                                          tag: "span",
                                          size: "small",
                                          bold: ""
                                        }, {
                                          default: _withCtx(() => [
                                            _createTextVNode(_toDisplayString(trigger.method), 1 /* TEXT */)
                                          ]),
                                          _: 2 /* DYNAMIC */
                                        }, 1024 /* DYNAMIC_SLOTS */),
                                        _createVNode($setup["N8nText"], {
                                          tag: "code",
                                          size: "small",
                                          "data-test-id": "instance-ai-test-listener-url"
                                        }, {
                                          default: _withCtx(() => [
                                            _createTextVNode(_toDisplayString(trigger.url), 1 /* TEXT */)
                                          ]),
                                          _: 2 /* DYNAMIC */
                                        }, 1024 /* DYNAMIC_SLOTS */)
                                      ], 2 /* CLASS */))
                                    }), 128 /* KEYED_FRAGMENT */)),
                                    _createVNode($setup["N8nText"], {
                                      tag: "div",
                                      size: "small",
                                      color: "text-light"
                                    }, {
                                      default: _withCtx(() => [
                                        _createTextVNode(_toDisplayString($setup.i18n.baseText('instanceAi.testListener.deadline', {
									interpolate: {
										time: $setup.formatDeadline(chunk.item.toolCall.confirmation.testListener.deadlineAt),
									},
								})), 1 /* TEXT */)
                                      ]),
                                      _: 2 /* DYNAMIC */
                                    }, 1024 /* DYNAMIC_SLOTS */),
                                    _createElementVNode("div", {
                                      class: _normalizeClass(_ctx.$style.continueRow)
                                    }, [
                                      _createVNode($setup["N8nButton"], {
                                        "data-test-id": "instance-ai-test-listener-cancel",
                                        size: "medium",
                                        variant: "outline",
                                        onClick: ($event: any) => ($setup.settleTestListener(chunk.item.toolCall.confirmation, { approved: false }))
                                      }, {
                                        default: _withCtx(() => [
                                          _createTextVNode(_toDisplayString($setup.i18n.baseText('instanceAi.testListener.cancel')), 1 /* TEXT */)
                                        ]),
                                        _: 1 /* STABLE */
                                      }, 8 /* PROPS */, ["onClick"]),
                                      _createVNode($setup["N8nButton"], {
                                        "data-test-id": "instance-ai-test-listener-sent",
                                        size: "medium",
                                        variant: "solid",
                                        onClick: ($event: any) => ($setup.settleTestListener(chunk.item.toolCall.confirmation, { approved: true }))
                                      }, {
                                        default: _withCtx(() => [
                                          _createTextVNode(_toDisplayString($setup.i18n.baseText('instanceAi.testListener.sent')), 1 /* TEXT */)
                                        ]),
                                        _: 1 /* STABLE */
                                      }, 8 /* PROPS */, ["onClick"])
                                    ], 2 /* CLASS */)
                                  ]),
                                  _: 2 /* DYNAMIC */
                                }, 1032 /* PROPS, DYNAMIC_SLOTS */, ["class"])
                              ]))
                            : (
						chunk.item.toolCall.confirmation.inputType === 'resource-decision' &&
						chunk.item.toolCall.confirmation.resourceDecision
					)
                              ? (_openBlock(), _createBlock($setup["GatewayResourceDecision"], {
                                  key: 'rd-' + chunk.item.toolCall.confirmation.requestId,
                                  "data-test-id": "instance-ai-gateway-confirmation-panel",
                                  "request-id": chunk.item.toolCall.confirmation.requestId,
                                  resource: chunk.item.toolCall.confirmation.resourceDecision.resource,
                                  description: chunk.item.toolCall.confirmation.resourceDecision.description,
                                  options: chunk.item.toolCall.confirmation.resourceDecision.options
                                }, null, 8 /* PROPS */, ["request-id", "resource", "description", "options"]))
                              : (chunk.item.toolCall.confirmation.channelConfig)
                                ? (_openBlock(), _createBlock($setup["InstanceAiChannelSetup"], {
                                    key: 'channel-' + chunk.item.toolCall.confirmation.requestId,
                                    "request-id": chunk.item.toolCall.confirmation.requestId,
                                    "integration-type": chunk.item.toolCall.confirmation.channelConfig.integrationType,
                                    "agent-id": chunk.item.toolCall.confirmation.channelConfig.agentId,
                                    "project-id": chunk.item.toolCall.confirmation.projectId ?? ''
                                  }, null, 8 /* PROPS */, ["request-id", "integration-type", "agent-id", "project-id"]))
                                : _createCommentVNode("v-if", true)
                ], 64 /* STABLE_FRAGMENT */))
              : (chunk.item.toolCall.confirmation.domainAccess)
                ? (_openBlock(), _createBlock($setup["DomainAccessApproval"], {
                    key: 'floating-' + chunk.item.toolCall.confirmation.requestId,
                    "data-test-id": "instance-ai-confirmation-panel",
                    "request-id": chunk.item.toolCall.confirmation.requestId,
                    url: chunk.item.toolCall.confirmation.domainAccess.url,
                    host: chunk.item.toolCall.confirmation.domainAccess.host,
                    severity: chunk.item.toolCall.confirmation.severity
                  }, null, 8 /* PROPS */, ["request-id", "url", "host", "severity"]))
                : (chunk.item.toolCall.confirmation.webSearch)
                  ? (_openBlock(), _createBlock($setup["DomainAccessApproval"], {
                      key: 'floating-' + chunk.item.toolCall.confirmation.requestId,
                      "data-test-id": "instance-ai-confirmation-panel",
                      "request-id": chunk.item.toolCall.confirmation.requestId,
                      query: chunk.item.toolCall.confirmation.webSearch.query,
                      severity: chunk.item.toolCall.confirmation.severity
                    }, null, 8 /* PROPS */, ["request-id", "query", "severity"]))
                  : (_openBlock(), _createBlock($setup["N8nApprovalCard"], {
                      key: 'floating-' + chunk.item.toolCall.confirmation.requestId,
                      "data-test-id": "instance-ai-confirmation-panel",
                      title: $setup.buildApprovalTitle(chunk.item),
                      labels: $setup.approvalLabels,
                      description: $setup.buildApprovalSubtitle(chunk.item),
                      args: chunk.item.toolCall.confirmation.targetApproval?.args,
                      "description-label": $setup.i18n.baseText('instanceAi.confirmation.details'),
                      options: $setup.credentialDestinationOptions(chunk.item),
                      "supports-session-approval": $setup.canAlwaysAllow(chunk.item),
                      destructive: $setup.isDestructive(chunk.item),
                      onSelect: (key) => $setup.handleApprovalSelect(chunk.item, key)
                    }, _createSlots({ _: 2 /* DYNAMIC */ }, [
                      (
						$setup.settingsStore.isInstanceAiSetupPanelEnabled &&
						chunk.item.toolCall.confirmation.credentialDestination
					)
                        ? {
                            name: "description",
                            fn: _withCtx(() => [
                              _createVNode($setup["N8nText"], {
                                tag: "p",
                                size: "small",
                                class: _normalizeClass(_ctx.$style.credentialDescription)
                              }, {
                                default: _withCtx(() => [
                                  _createTextVNode(_toDisplayString($setup.buildApprovalSubtitle(chunk.item)), 1 /* TEXT */)
                                ]),
                                _: 2 /* DYNAMIC */
                              }, 1032 /* PROPS, DYNAMIC_SLOTS */, ["class"])
                            ]),
                            key: "0"
                          }
                        : undefined
                    ]), 1032 /* PROPS, DYNAMIC_SLOTS */, ["title", "labels", "description", "args", "description-label", "options", "supports-session-approval", "destructive", "onSelect"]))
        ], 64 /* STABLE_FRAGMENT */))
      }), 128 /* KEYED_FRAGMENT */))
    ]),
    _: 1 /* STABLE */
  }))
}