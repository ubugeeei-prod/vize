import { openBlock as _openBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref } from "vue";
const _hoisted_1 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-settings" });
const _hoisted_2 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-notes" });
import { computed, onMounted, ref, useCssModule } from "vue";
import { misskeyApi } from "@/utility/misskey-api.js";
import { emptyStrToEmptyArray, emptyStrToNull, roleIdsParser } from "@/pages/admin/custom-emojis-manager.impl.js";
import MkGrid from "@/components/grid/MkGrid.vue";
import { i18n } from "@/i18n.js";
import MkSelect from "@/components/MkSelect.vue";
import MkSwitch from "@/components/MkSwitch.vue";
import MkFolder from "@/components/MkFolder.vue";
import MkButton from "@/components/MkButton.vue";
import * as os from "@/os.js";
import { validators } from "@/components/grid/cell-validators.js";
import { chooseDriveFile, chooseFileFromPcAndUpload } from "@/utility/drive.js";
import XRegisterLogs from "@/pages/admin/custom-emojis-manager.logs.vue";
import { copyGridDataToClipboard } from "@/components/grid/grid-utils.js";
import { useMkSelect } from "@/composables/use-mkselect.js";
import { prefer } from "@/preferences.js";
const MAXIMUM_EMOJI_REGISTER_COUNT = 100;
export default {
  __name: "custom-emojis-manager.register",
  setup(__props) {
    function setupGrid() {
      const $style = useCssModule();
      const required = validators.required();
      const regex = validators.regex(/^[a-zA-Z0-9_]+$/);
      const unique = validators.unique();
      function removeRows(rows) {
        const idxes = [...new Set(rows.map((it) => it.index))];
        gridItems.value = gridItems.value.filter((_, i) => !idxes.includes(i));
      }
      return {
        row: {
          showNumber: true,
          selectable: true,
          minimumDefinitionCount: 100,
          styleRules: [{
            // 1つでもバリデーションエラーがあれば行全体をエラー表示する
            condition: ({ cells }) => cells.some((it) => !it.violation.valid),
            applyStyle: { className: $style.violationRow }
          }],
          // 行のコンテキストメニュー設定
          contextMenuFactory: (row, context) => {
            return [{
              type: "button",
              text: i18n.ts._customEmojisManager._gridCommon.copySelectionRows,
              icon: "ti ti-copy",
              action: () => copyGridDataToClipboard(gridItems, context)
            }, {
              type: "button",
              text: i18n.ts._customEmojisManager._gridCommon.deleteSelectionRows,
              icon: "ti ti-trash",
              action: () => removeRows(context.rangedRows)
            }];
          },
          events: { delete(rows) {
            removeRows(rows);
          } }
        },
        cols: [
          {
            bindTo: "url",
            icon: "ti-icons",
            type: "image",
            editable: false,
            width: "auto",
            validators: [required]
          },
          {
            bindTo: "name",
            title: "name",
            type: "text",
            editable: true,
            width: 140,
            validators: [
              required,
              regex,
              unique
            ]
          },
          {
            bindTo: "category",
            title: "category",
            type: "text",
            editable: true,
            width: 140
          },
          {
            bindTo: "aliases",
            title: "aliases",
            type: "text",
            editable: true,
            width: 140
          },
          {
            bindTo: "license",
            title: "license",
            type: "text",
            editable: true,
            width: 140
          },
          {
            bindTo: "isSensitive",
            title: "sensitive",
            type: "boolean",
            editable: true,
            width: 90
          },
          {
            bindTo: "localOnly",
            title: "localOnly",
            type: "boolean",
            editable: true,
            width: 90
          },
          {
            bindTo: "roleIdsThatCanBeUsedThisEmojiAsReaction",
            title: "role",
            type: "text",
            editable: true,
            width: 140,
            valueTransformer: (row) => {
              // バックエンドからからはIDと名前のペア配列で受け取るが、表示にIDがあると煩雑なので名前だけにする
              return gridItems.value[row.index].roleIdsThatCanBeUsedThisEmojiAsReaction.map((it) => it.name).join(",");
            },
            customValueEditor: async (row) => {
              // ID直記入は体験的に最悪なのでモーダルを使って入力する
              const current = gridItems.value[row.index].roleIdsThatCanBeUsedThisEmojiAsReaction;
              const result = await os.selectRole({
                initialRoleIds: current.map((it) => it.id),
                title: i18n.ts.rolesThatCanBeUsedThisEmojiAsReaction,
                infoMessage: i18n.ts.rolesThatCanBeUsedThisEmojiAsReactionEmptyDescription,
                publicOnly: true
              });
              if (result.canceled) {
                return current;
              }
              const transform = result.result.map((it) => ({
                id: it.id,
                name: it.name
              }));
              gridItems.value[row.index].roleIdsThatCanBeUsedThisEmojiAsReaction = transform;
              return transform;
            },
            events: {
              paste: roleIdsParser,
              delete(cell) {
                // デフォルトはundefinedになるが、このプロパティは空配列にしたい
                gridItems.value[cell.row.index].roleIdsThatCanBeUsedThisEmojiAsReaction = [];
              }
            }
          },
          {
            bindTo: "type",
            type: "text",
            editable: false,
            width: 90
          }
        ],
        cells: { 
        // セルのコンテキストメニュー設定
contextMenuFactory: (col, row, value, context) => {
          return [{
            type: "button",
            text: i18n.ts._customEmojisManager._gridCommon.copySelectionRanges,
            icon: "ti ti-copy",
            action: () => copyGridDataToClipboard(gridItems, context)
          }, {
            type: "button",
            text: i18n.ts._customEmojisManager._gridCommon.deleteSelectionRanges,
            icon: "ti ti-trash",
            action: () => removeRows(context.rangedCells.map((it) => it.row))
          }];
        } }
      };
    }
    const uploadFolders = ref([]);
    const gridItems = ref([]);
    const { model: selectedFolderId, def: selectedFolderIdDef } = useMkSelect({
      items: computed(() => uploadFolders.value.map((folder) => ({
        label: folder.name,
        value: folder.id || ""
      }))),
      initialValue: prefer.s.uploadFolder
    });
    const directoryToCategory = ref(false);
    const registerButtonDisabled = ref(false);
    const requestLogs = ref([]);
    const isDragOver = ref(false);
    async function onRegistryClicked() {
      const dialogSelection = await os.confirm({
        type: "info",
        text: i18n.tsx._customEmojisManager._local._register.confirmRegisterEmojisDescription({ count: MAXIMUM_EMOJI_REGISTER_COUNT })
      });
      if (dialogSelection.canceled) {
        return;
      }
      const items = gridItems.value;
      const upload = () => {
        return items.slice(0, MAXIMUM_EMOJI_REGISTER_COUNT).map((item) => misskeyApi("admin/emoji/add", {
          name: item.name,
          category: emptyStrToNull(item.category),
          aliases: emptyStrToEmptyArray(item.aliases),
          license: emptyStrToNull(item.license),
          isSensitive: item.isSensitive,
          localOnly: item.localOnly,
          roleIdsThatCanBeUsedThisEmojiAsReaction: item.roleIdsThatCanBeUsedThisEmojiAsReaction.map((it) => it.id),
          fileId: item.fileId
        }).then(() => ({
          item,
          success: true,
          err: undefined
        })).catch((err) => ({
          item,
          success: false,
          err
        })));
      };
      const result = await os.promiseDialog(Promise.all(upload()));
      const failedItems = result.filter((it) => !it.success);
      if (failedItems.length > 0) {
        await os.alert({
          type: "error",
          title: i18n.ts.somethingHappened,
          text: i18n.ts._customEmojisManager._gridCommon.alertEmojisRegisterFailedDescription
        });
      }
      requestLogs.value = result.map((it) => ({
        failed: !it.success,
        url: it.item.url,
        name: it.item.name,
        error: it.err ? JSON.stringify(it.err) : undefined
      }));
      // 登録に成功したものは一覧から除く
      const successItems = result.filter((it) => it.success).map((it) => it.item);
      gridItems.value = gridItems.value.filter((it) => !successItems.includes(it));
    }
    async function onClearClicked() {
      const result = await os.confirm({
        type: "warning",
        text: i18n.ts._customEmojisManager._local._register.confirmClearEmojisDescription
      });
      if (!result.canceled) {
        gridItems.value = [];
      }
    }
    async function onFileSelectClicked() {
      const driveFiles = await chooseFileFromPcAndUpload({
        multiple: true,
        folderId: selectedFolderId.value
      });
      gridItems.value.push(...driveFiles.map(fromDriveFile));
    }
    async function onDriveSelectClicked() {
      const driveFiles = await chooseDriveFile({ multiple: true });
      gridItems.value.push(...driveFiles.map(fromDriveFile));
    }
    function onGridEvent(event) {
      switch (event.type) {
        case "cell-validation":
          onGridCellValidation(event);
          break;
        case "cell-value-change":
          onGridCellValueChange(event);
          break;
      }
    }
    function onGridCellValidation(event) {
      registerButtonDisabled.value = event.all.filter((it) => !it.valid).length > 0;
    }
    function onGridCellValueChange(event) {
      const { row, column, newValue } = event;
      if (gridItems.value.length > row.index && column.setting.bindTo in gridItems.value[row.index]) {
        gridItems.value[row.index][column.setting.bindTo] = newValue;
      }
    }
    function fromDriveFile(it) {
      return {
        fileId: it.id,
        url: it.url,
        name: it.name.replace(/(\.[a-zA-Z0-9]+)+$/, "").replaceAll("-", "_").replaceAll(" ", "_"),
        host: "",
        category: "",
        aliases: "",
        license: "",
        isSensitive: it.isSensitive,
        localOnly: false,
        roleIdsThatCanBeUsedThisEmojiAsReaction: [],
        type: it.type
      };
    }
    async function refreshUploadFolders() {
      const result = await misskeyApi("drive/folders", {});
      uploadFolders.value = Array.of({ name: "-" }, ...result);
    }
    onMounted(async () => {
      await refreshUploadFolders();
    });
    return (_ctx, _cache) => {
      return _openBlock(), _createElementBlock("div", { class: "_spacer" }, [_createElementVNode("div", { class: "_gaps" }, [
        _createVNode(MkFolder, null, {
          icon: _withCtx(() => [_hoisted_1]),
          label: _withCtx(() => [_createTextVNode(
            _toDisplayString(_unref(i18n).ts._customEmojisManager._local._register.uploadSettingTitle),
            1
            /* TEXT */
          )]),
          caption: _withCtx(() => [_createTextVNode(
            _toDisplayString(_unref(i18n).ts._customEmojisManager._local._register.uploadSettingDescription),
            1
            /* TEXT */
          )]),
          default: _withCtx(() => [_createElementVNode("div", { class: "_gaps" }, [_createVNode(MkSelect, {
            items: _unref(selectedFolderIdDef),
            modelValue: _unref(selectedFolderId),
            "onUpdate:modelValue": _cache[0] || (_cache[0] = ($event) => selectedFolderId.value = $event)
          }, {
            label: _withCtx(() => [_createTextVNode(
              _toDisplayString(_unref(i18n).ts.uploadFolder),
              1
              /* TEXT */
            )]),
            _: 1
          }, 8, ["items", "modelValue"]), _createVNode(MkSwitch, {
            modelValue: directoryToCategory.value,
            "onUpdate:modelValue": _cache[1] || (_cache[1] = ($event) => directoryToCategory.value = $event)
          }, {
            label: _withCtx(() => [_createTextVNode(
              _toDisplayString(_unref(i18n).ts._customEmojisManager._local._register.directoryToCategoryLabel),
              1
              /* TEXT */
            )]),
            caption: _withCtx(() => [_createTextVNode(
              _toDisplayString(_unref(i18n).ts._customEmojisManager._local._register.directoryToCategoryCaption),
              1
              /* TEXT */
            )]),
            _: 1
          }, 8, ["modelValue"])])]),
          _: 1
        }),
        _createVNode(MkFolder, null, {
          icon: _withCtx(() => [_hoisted_2]),
          label: _withCtx(() => [_createTextVNode(
            _toDisplayString(_unref(i18n).ts._customEmojisManager._gridCommon.registrationLogs),
            1
            /* TEXT */
          )]),
          caption: _withCtx(() => [_createTextVNode(
            _toDisplayString(_unref(i18n).ts._customEmojisManager._gridCommon.registrationLogsCaption),
            1
            /* TEXT */
          )]),
          default: _withCtx(() => [_createVNode(XRegisterLogs, { logs: requestLogs.value }, null, 8, ["logs"])]),
          _: 1
        }),
        _createElementVNode("div", { class: "_buttonsCenter" }, [_createVNode(MkButton, {
          primary: "",
          rounded: "",
          onClick: onFileSelectClicked
        }, {
          default: _withCtx(() => [_createTextVNode(
            _toDisplayString(_unref(i18n).ts.upload),
            1
            /* TEXT */
          )]),
          _: 1
        }), _createVNode(MkButton, {
          primary: "",
          rounded: "",
          onClick: onDriveSelectClicked
        }, {
          default: _withCtx(() => [_createTextVNode(
            _toDisplayString(_unref(i18n).ts.fromDrive),
            1
            /* TEXT */
          )]),
          _: 1
        })]),
        gridItems.value.length > 0 ? (_openBlock(), _createElementBlock(
          "div",
          {
            key: 0,
            class: _normalizeClass(_ctx.$style.gridArea)
          },
          [_createVNode(MkGrid, {
            data: gridItems.value,
            settings: setupGrid(),
            onEvent: onGridEvent
          }, null, 8, ["data", "settings"])],
          2
          /* CLASS */
        )) : _createCommentVNode("v-if", true),
        gridItems.value.length > 0 ? (_openBlock(), _createElementBlock(
          "div",
          {
            key: 0,
            class: _normalizeClass(_ctx.$style.footer)
          },
          [_createVNode(MkButton, {
            primary: "",
            disabled: registerButtonDisabled.value,
            onClick: onRegistryClicked
          }, {
            default: _withCtx(() => [_createTextVNode(
              _toDisplayString(_unref(i18n).ts.registration),
              1
              /* TEXT */
            )]),
            _: 1
          }, 8, ["disabled"]), _createVNode(MkButton, { onClick: onClearClicked }, {
            default: _withCtx(() => [_createTextVNode(
              _toDisplayString(_unref(i18n).ts.clear),
              1
              /* TEXT */
            )]),
            _: 1
          })],
          2
          /* CLASS */
        )) : _createCommentVNode("v-if", true)
      ])]);
    };
  }
};
