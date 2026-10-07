const _Vue = Vue

return function render(_ctx, _cache) {
  with (_ctx) {
    const { renderList: _renderList, Fragment: _Fragment, openBlock: _openBlock, createElementBlock: _createElementBlock, resolveComponent: _resolveComponent, createBlock: _createBlock, createCommentVNode: _createCommentVNode, toDisplayString: _toDisplayString, createTextVNode: _createTextVNode, withCtx: _withCtx, createVNode: _createVNode, withKeys: _withKeys, normalizeClass: _normalizeClass, createElementVNode: _createElementVNode, createSlots: _createSlots, TransitionGroup: _TransitionGroup } = _Vue

    const _component_InstanceAiQuestions = _resolveComponent("InstanceAiQuestions")
    const _component_InstanceAiWorkflowSetup = _resolveComponent("InstanceAiWorkflowSetup")
    const _component_InstanceAiCredentialSetup = _resolveComponent("InstanceAiCredentialSetup")
    const _component_N8nText = _resolveComponent("N8nText")
    const _component_N8nInput = _resolveComponent("N8nInput")
    const _component_N8nButton = _resolveComponent("N8nButton")
    const _component_N8nCard = _resolveComponent("N8nCard")
    const _component_GatewayResourceDecision = _resolveComponent("GatewayResourceDecision")
    const _component_InstanceAiChannelSetup = _resolveComponent("InstanceAiChannelSetup")
    const _component_DomainAccessApproval = _resolveComponent("DomainAccessApproval")
    const _component_N8nApprovalCard = _resolveComponent("N8nApprovalCard")

    return (_openBlock(), _createBlock(_TransitionGroup, { name: "confirmation-slide" }, {
      default: _withCtx(() => [
        (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(chunks, (chunk) => {
          return (_openBlock(), _createElementBlock(_Fragment, { key: chunk.item.toolCall.confirmation.requestId }, [
            (
					chunk.type === 'floating' &&
					chunk.item.toolCall.confirmation.inputType === 'questions' &&
					chunk.item.toolCall.confirmation.questions
				)
              ? (_openBlock(), _createBlock(_component_InstanceAiQuestions, {
                  key: 'q-' + chunk.item.toolCall.confirmation.requestId,
                  questions: chunk.item.toolCall.confirmation.questions!,
                  "intro-message": chunk.item.toolCall.confirmation.introMessage,
                  onSubmit: (answers) => handleQuestionsSubmit(chunk.item.toolCall.confirmation, answers)
                }, null, 8 /* PROPS */, ["questions", "intro-message", "onSubmit"]))
              : (chunk.type === 'standalone')
                ? (_openBlock(), _createElementBlock(_Fragment, { key: 1 }, [
                    (chunk.item.toolCall.confirmation.setupRequests?.length)
                      ? (_openBlock(), _createBlock(_component_InstanceAiWorkflowSetup, {
                          key: 'setup-' + chunk.item.toolCall.confirmation.requestId,
                          "request-id": chunk.item.toolCall.confirmation.requestId,
                          "setup-requests": chunk.item.toolCall.confirmation.setupRequests!,
                          "project-id": chunk.item.toolCall.confirmation.projectId ?? thread.projectId,
                          "credential-flow": chunk.item.toolCall.confirmation.credentialFlow,
                          "workflow-id": chunk.item.toolCall.confirmation.workflowId
                        }, null, 8 /* PROPS */, ["request-id", "setup-requests", "project-id", "credential-flow", "workflow-id"]))
                      : (chunk.item.toolCall.confirmation.credentialRequests?.length)
                        ? (_openBlock(), _createBlock(_component_InstanceAiCredentialSetup, {
                            key: 'cred-' + chunk.item.toolCall.confirmation.requestId,
                            "request-id": chunk.item.toolCall.confirmation.requestId,
                            "credential-requests": chunk.item.toolCall.confirmation.credentialRequests!,
                            message: chunk.item.toolCall.confirmation.message,
                            "project-id": chunk.item.toolCall.confirmation.projectId ?? thread.projectId,
                            "credential-flow": chunk.item.toolCall.confirmation.credentialFlow,
                            "require-user-selection": chunk.item.toolCall.confirmation.requireUserSelection
                          }, null, 8 /* PROPS */, ["request-id", "credential-requests", "message", "project-id", "credential-flow", "require-user-selection"]))
                        : (chunk.item.toolCall.confirmation.inputType === 'text')
                          ? (_openBlock(), _createElementBlock("div", { key: 'text-' + chunk.item.toolCall.confirmation.requestId }, [
                              _createVNode(_component_N8nCard, {
                                class: _normalizeClass($style.textCard)
                              }, {
                                default: _withCtx(() => [
                                  _createVNode(_component_N8nText, { tag: "div" }, {
                                    default: _withCtx(() => [
                                      _createTextVNode(_toDisplayString(chunk.item.toolCall.confirmation!.message), 1 /* TEXT */)
                                    ]),
                                    _: 2 /* DYNAMIC */
                                  }, 1024 /* DYNAMIC_SLOTS */),
                                  _createElementVNode("div", {
                                    class: _normalizeClass($style.textInputRow)
                                  }, [
                                    _createVNode(_component_N8nInput, {
                                      modelValue: textInputValues[chunk.item.toolCall.confirmation!.requestId],
                                      "onUpdate:modelValue": $event => ((textInputValues[chunk.item.toolCall.confirmation!.requestId]) = $event),
                                      type: "text",
                                      size: "small",
                                      placeholder: i18n.baseText('instanceAi.askUser.placeholder'),
                                      onKeydown: _withKeys($event => (handleTextSubmit(chunk.item.toolCall.confirmation)), ["enter"])
                                    }, null, 8 /* PROPS */, ["modelValue", "onUpdate:modelValue", "placeholder", "onKeydown"]),
                                    (!(textInputValues[chunk.item.toolCall.confirmation.requestId] ?? '').trim())
                                      ? (_openBlock(), _createBlock(_component_N8nButton, {
                                          key: 0,
                                          size: "medium",
                                          variant: "outline",
                                          onClick: $event => (handleTextSkip(chunk.item.toolCall.confirmation))
                                        }, {
                                          default: _withCtx(() => [
                                            _createTextVNode(_toDisplayString(i18n.baseText('instanceAi.askUser.skip')), 1 /* TEXT */)
                                          ]),
                                          _: 2 /* DYNAMIC */
                                        }, 1032 /* PROPS, DYNAMIC_SLOTS */, ["onClick"]))
                                      : _createCommentVNode("v-if", true),
                                    _createVNode(_component_N8nButton, {
                                      size: "medium",
                                      variant: "solid",
                                      disabled: 
									!(textInputValues[chunk.item.toolCall.confirmation.requestId] ?? '').trim()
								,
                                      onClick: $event => (handleTextSubmit(chunk.item.toolCall.confirmation))
                                    }, {
                                      default: _withCtx(() => [
                                        _createTextVNode(_toDisplayString(i18n.baseText('instanceAi.askUser.submit')), 1 /* TEXT */)
                                      ]),
                                      _: 2 /* DYNAMIC */
                                    }, 1032 /* PROPS, DYNAMIC_SLOTS */, ["disabled", "onClick"])
                                  ], 2 /* CLASS */)
                                ]),
                                _: 2 /* DYNAMIC */
                              }, 1032 /* PROPS, DYNAMIC_SLOTS */, ["class"])
                            ]))
                          : (chunk.item.toolCall.confirmation.inputType === 'continue')
                            ? (_openBlock(), _createElementBlock("div", { key: 'continue-' + chunk.item.toolCall.confirmation.requestId }, [
                                _createVNode(_component_N8nCard, {
                                  class: _normalizeClass($style.textCard)
                                }, {
                                  default: _withCtx(() => [
                                    _createVNode(_component_N8nText, { tag: "div" }, {
                                      default: _withCtx(() => [
                                        _createTextVNode(_toDisplayString(chunk.item.toolCall.confirmation!.message), 1 /* TEXT */)
                                      ]),
                                      _: 2 /* DYNAMIC */
                                    }, 1024 /* DYNAMIC_SLOTS */),
                                    _createElementVNode("div", {
                                      class: _normalizeClass($style.continueRow)
                                    }, [
                                      _createVNode(_component_N8nButton, {
                                        "data-test-id": "instance-ai-panel-continue",
                                        size: "medium",
                                        variant: "solid",
                                        onClick: $event => (handleContinue(chunk.item.toolCall.confirmation))
                                      }, {
                                        default: _withCtx(() => [
                                          _createTextVNode(_toDisplayString(i18n.baseText('instanceAi.confirmation.continue')), 1 /* TEXT */)
                                        ]),
                                        _: 2 /* DYNAMIC */
                                      }, 1032 /* PROPS, DYNAMIC_SLOTS */, ["onClick"])
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
                                  _createVNode(_component_N8nCard, {
                                    class: _normalizeClass($style.textCard)
                                  }, {
                                    default: _withCtx(() => [
                                      _createVNode(_component_N8nText, { tag: "div" }, {
                                        default: _withCtx(() => [
                                          _createTextVNode(_toDisplayString(chunk.item.toolCall.confirmation.message), 1 /* TEXT */)
                                        ]),
                                        _: 2 /* DYNAMIC */
                                      }, 1024 /* DYNAMIC_SLOTS */),
                                      (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(chunk.item.toolCall.confirmation.testListener.triggers, (trigger) => {
                                        return (_openBlock(), _createElementBlock("div", {
                                          key: trigger.nodeName,
                                          class: _normalizeClass($style.testListenerUrl)
                                        }, [
                                          _createVNode(_component_N8nText, {
                                            tag: "span",
                                            size: "small",
                                            bold: ""
                                          }, {
                                            default: _withCtx(() => [
                                              _createTextVNode(_toDisplayString(trigger.method), 1 /* TEXT */)
                                            ]),
                                            _: 2 /* DYNAMIC */
                                          }, 1024 /* DYNAMIC_SLOTS */),
                                          _createVNode(_component_N8nText, {
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
                                      _createVNode(_component_N8nText, {
                                        tag: "div",
                                        size: "small",
                                        color: "text-light"
                                      }, {
                                        default: _withCtx(() => [
                                          _createTextVNode(_toDisplayString(i18n.baseText('instanceAi.testListener.deadline', {
									interpolate: {
										time: formatDeadline(chunk.item.toolCall.confirmation.testListener.deadlineAt),
									},
								})), 1 /* TEXT */)
                                        ]),
                                        _: 2 /* DYNAMIC */
                                      }, 1024 /* DYNAMIC_SLOTS */),
                                      _createElementVNode("div", {
                                        class: _normalizeClass($style.continueRow)
                                      }, [
                                        _createVNode(_component_N8nButton, {
                                          "data-test-id": "instance-ai-test-listener-cancel",
                                          size: "medium",
                                          variant: "outline",
                                          onClick: $event => (settleTestListener(chunk.item.toolCall.confirmation, { approved: false }))
                                        }, {
                                          default: _withCtx(() => [
                                            _createTextVNode(_toDisplayString(i18n.baseText('instanceAi.testListener.cancel')), 1 /* TEXT */)
                                          ]),
                                          _: 2 /* DYNAMIC */
                                        }, 1032 /* PROPS, DYNAMIC_SLOTS */, ["onClick"]),
                                        _createVNode(_component_N8nButton, {
                                          "data-test-id": "instance-ai-test-listener-sent",
                                          size: "medium",
                                          variant: "solid",
                                          onClick: $event => (settleTestListener(chunk.item.toolCall.confirmation, { approved: true }))
                                        }, {
                                          default: _withCtx(() => [
                                            _createTextVNode(_toDisplayString(i18n.baseText('instanceAi.testListener.sent')), 1 /* TEXT */)
                                          ]),
                                          _: 2 /* DYNAMIC */
                                        }, 1032 /* PROPS, DYNAMIC_SLOTS */, ["onClick"])
                                      ], 2 /* CLASS */)
                                    ]),
                                    _: 2 /* DYNAMIC */
                                  }, 1032 /* PROPS, DYNAMIC_SLOTS */, ["class"])
                                ]))
                              : (
						chunk.item.toolCall.confirmation.inputType === 'resource-decision' &&
						chunk.item.toolCall.confirmation.resourceDecision
					)
                                ? (_openBlock(), _createBlock(_component_GatewayResourceDecision, {
                                    key: 'rd-' + chunk.item.toolCall.confirmation.requestId,
                                    "data-test-id": "instance-ai-gateway-confirmation-panel",
                                    "request-id": chunk.item.toolCall.confirmation.requestId,
                                    resource: chunk.item.toolCall.confirmation.resourceDecision.resource,
                                    description: chunk.item.toolCall.confirmation.resourceDecision.description,
                                    options: chunk.item.toolCall.confirmation.resourceDecision.options
                                  }, null, 8 /* PROPS */, ["request-id", "resource", "description", "options"]))
                                : (chunk.item.toolCall.confirmation.channelConfig)
                                  ? (_openBlock(), _createBlock(_component_InstanceAiChannelSetup, {
                                      key: 'channel-' + chunk.item.toolCall.confirmation.requestId,
                                      "request-id": chunk.item.toolCall.confirmation.requestId,
                                      "integration-type": chunk.item.toolCall.confirmation.channelConfig.integrationType,
                                      "agent-id": chunk.item.toolCall.confirmation.channelConfig.agentId,
                                      "project-id": chunk.item.toolCall.confirmation.projectId ?? ''
                                    }, null, 8 /* PROPS */, ["request-id", "integration-type", "agent-id", "project-id"]))
                                  : _createCommentVNode("v-if", true)
                  ], 64 /* STABLE_FRAGMENT */))
                : (chunk.item.toolCall.confirmation.domainAccess)
                  ? (_openBlock(), _createBlock(_component_DomainAccessApproval, {
                      key: 'floating-' + chunk.item.toolCall.confirmation.requestId,
                      "data-test-id": "instance-ai-confirmation-panel",
                      "request-id": chunk.item.toolCall.confirmation.requestId,
                      url: chunk.item.toolCall.confirmation.domainAccess.url,
                      host: chunk.item.toolCall.confirmation.domainAccess.host,
                      severity: chunk.item.toolCall.confirmation.severity
                    }, null, 8 /* PROPS */, ["request-id", "url", "host", "severity"]))
                  : (chunk.item.toolCall.confirmation.webSearch)
                    ? (_openBlock(), _createBlock(_component_DomainAccessApproval, {
                        key: 'floating-' + chunk.item.toolCall.confirmation.requestId,
                        "data-test-id": "instance-ai-confirmation-panel",
                        "request-id": chunk.item.toolCall.confirmation.requestId,
                        query: chunk.item.toolCall.confirmation.webSearch.query,
                        severity: chunk.item.toolCall.confirmation.severity
                      }, null, 8 /* PROPS */, ["request-id", "query", "severity"]))
                    : (_openBlock(), _createBlock(_component_N8nApprovalCard, {
                        key: 'floating-' + chunk.item.toolCall.confirmation.requestId,
                        "data-test-id": "instance-ai-confirmation-panel",
                        title: buildApprovalTitle(chunk.item),
                        labels: approvalLabels,
                        description: buildApprovalSubtitle(chunk.item),
                        args: chunk.item.toolCall.confirmation.targetApproval?.args,
                        "description-label": i18n.baseText('instanceAi.confirmation.details'),
                        options: credentialDestinationOptions(chunk.item),
                        "supports-session-approval": canAlwaysAllow(chunk.item),
                        destructive: isDestructive(chunk.item),
                        onSelect: (key) => handleApprovalSelect(chunk.item, key)
                      }, _createSlots({ _: 2 /* DYNAMIC */ }, [
                        (
						settingsStore.isInstanceAiSetupPanelEnabled &&
						chunk.item.toolCall.confirmation.credentialDestination
					)
                          ? {
                              name: "description",
                              fn: _withCtx(() => [
                                _createVNode(_component_N8nText, {
                                  tag: "p",
                                  size: "small",
                                  class: _normalizeClass($style.credentialDescription)
                                }, {
                                  default: _withCtx(() => [
                                    _createTextVNode(_toDisplayString(buildApprovalSubtitle(chunk.item)), 1 /* TEXT */)
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
}