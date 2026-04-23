import { Fragment as _Fragment, TransitionGroup as _TransitionGroup, openBlock as _openBlock, createBlock as _createBlock, createElementBlock as _createElementBlock, createVNode as _createVNode, createElementVNode as _createElementVNode, createCommentVNode as _createCommentVNode, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, resolveDirective as _resolveDirective, withDirectives as _withDirectives, renderList as _renderList, toDisplayString as _toDisplayString, normalizeClass as _normalizeClass, withCtx as _withCtx, unref as _unref, vShow as _vShow, withModifiers as _withModifiers } from "vue";
const _hoisted_1 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-chevron-right" });
const _hoisted_2 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-chevron-right" });
const _hoisted_3 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-dots" });
const _hoisted_4 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-square" });
const _hoisted_5 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-checkbox" });
const _hoisted_6 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-square" });
const _hoisted_7 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-checkbox" });
const _hoisted_8 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-chevron-down" });
const _hoisted_9 = /* @__PURE__ */ _createElementVNode("i", { class: "ti ti-folder-symlink" });
import { onActivated, onBeforeUnmount, onMounted, ref, watch, computed, markRaw } from "vue";
import MkButton from "./MkButton.vue";
import XNavFolder from "@/components/MkDrive.navFolder.vue";
import XFolder from "@/components/MkDrive.folder.vue";
import XFile from "@/components/MkDrive.file.vue";
import * as os from "@/os.js";
import { misskeyApi } from "@/utility/misskey-api.js";
import { useStream } from "@/stream.js";
import { i18n } from "@/i18n.js";
import { claimAchievement } from "@/utility/achievements.js";
import { prefer } from "@/preferences.js";
import { chooseFileFromPcAndUpload, selectDriveFolder } from "@/utility/drive.js";
import { store } from "@/store.js";
import { makeDateGroupedTimelineComputedRef } from "@/utility/timeline-date-separate.js";
import { globalEvents, useGlobalEvent } from "@/events.js";
import { checkDragDataType, getDragData, setDragData } from "@/drag-and-drop.js";
import { getDriveFileMenu } from "@/utility/get-drive-file-menu.js";
import { Paginator } from "@/utility/paginator.js";
export default {
  __name: "MkDrive",
  props: {
    initialFolder: {
      type: null,
      required: false,
      default: null
    },
    type: {
      type: String,
      required: false
    },
    multiple: {
      type: Boolean,
      required: false,
      default: false
    },
    select: {
      type: [String, null],
      required: false,
      default: null
    },
    forceDisableInfiniteScroll: {
      type: Boolean,
      required: false,
      default: false
    }
  },
  emits: [
    "changeSelectedFiles",
    "changeSelectedFolders",
    "cd"
  ],
  setup(__props, { emit: __emit }) {
    const emit = __emit;
    const props = __props;
    const shouldEnableInfiniteScroll = computed(() => {
      return prefer.r.enableInfiniteScroll.value && !props.forceDisableInfiniteScroll;
    });
    const folder = ref(null);
    const hierarchyFolders = ref([]);
    // ドロップされようとしているか
    const draghover = ref(false);
    // 自身の所有するアイテムがドラッグをスタートさせたか
    // (自分自身の階層にドロップできないようにするためのフラグ)
    const isDragSource = ref(false);
    const isEditMode = ref(false);
    const selectedFiles = ref([]);
    const selectedFolders = ref([]);
    const isRootSelected = ref(false);
    watch(selectedFiles, () => {
      emit("changeSelectedFiles", selectedFiles.value);
    }, { deep: true });
    watch([selectedFolders, isRootSelected], () => {
      emit("changeSelectedFolders", isRootSelected.value ? [null, ...selectedFolders.value] : selectedFolders.value);
    });
    const fetching = ref(true);
    const sortModeSelect = ref("+createdAt");
    const filesPaginator = markRaw(new Paginator("drive/files", {
      limit: 30,
      canFetchDetection: "limit",
      params: () => ({
        folderId: folder.value ? folder.value.id : null,
        type: props.type,
        sort: ["-createdAt", "+createdAt"].includes(sortModeSelect.value) ? null : sortModeSelect.value
      })
    }));
    const foldersPaginator = markRaw(new Paginator("drive/folders", {
      limit: 30,
      canFetchDetection: "limit",
      params: () => ({ folderId: folder.value ? folder.value.id : null })
    }));
    const canFetchFiles = computed(() => !fetching.value && (filesPaginator.order.value === "oldest" ? filesPaginator.canFetchNewer.value : filesPaginator.canFetchOlder.value));
    async function fetchMoreFiles() {
      if (filesPaginator.order.value === "oldest") {
        filesPaginator.fetchNewer();
      } else {
        filesPaginator.fetchOlder();
      }
    }
    const filesTimeline = makeDateGroupedTimelineComputedRef(filesPaginator.items, "month");
    const shouldBeGroupedByDate = computed(() => ["+createdAt", "-createdAt"].includes(sortModeSelect.value));
    watch(folder, () => emit("cd", folder.value));
    watch(sortModeSelect, () => {
      initialize();
    });
    async function initialize() {
      fetching.value = true;
      await foldersPaginator.reload();
      filesPaginator.initialDirection = sortModeSelect.value === "-createdAt" ? "newer" : "older";
      filesPaginator.order.value = sortModeSelect.value === "-createdAt" ? "oldest" : "newest";
      await filesPaginator.reload();
      fetching.value = false;
    }
    function onStreamDriveFileCreated(file) {
      if (file.folderId === (folder.value?.id ?? null)) {
        filesPaginator.prepend(file);
      }
    }
    function onFileDragstart(file, ev) {
      if (isEditMode.value) {
        if (!selectedFiles.value.some((f) => f.id === file.id)) {
          selectedFiles.value.push(file);
        }
        if (ev.dataTransfer) {
          ev.dataTransfer.effectAllowed = "move";
          setDragData(ev, "driveFiles", selectedFiles.value);
        }
      }
      isDragSource.value = true;
    }
    function onDragover(ev) {
      if (!ev.dataTransfer) return;
      // ドラッグ元が自分自身の所有するアイテムだったら
      if (isDragSource.value) {
        // 自分自身にはドロップさせない
        ev.dataTransfer.dropEffect = "none";
        return;
      }
      const isFile = ev.dataTransfer.items[0].kind === "file";
      if (isFile || checkDragDataType(ev, ["driveFiles", "driveFolders"])) {
        switch (ev.dataTransfer.effectAllowed) {
          case "all":
          case "uninitialized":
          case "copy":
          case "copyLink":
          case "copyMove":
            ev.dataTransfer.dropEffect = "copy";
            break;
          case "linkMove":
          case "move":
            ev.dataTransfer.dropEffect = "move";
            break;
          default:
            ev.dataTransfer.dropEffect = "none";
            break;
        }
      } else {
        ev.dataTransfer.dropEffect = "none";
      }
      return false;
    }
    function onDragenter() {
      if (!isDragSource.value) draghover.value = true;
    }
    function onDragleave() {
      draghover.value = false;
    }
    function onDrop(ev) {
      draghover.value = false;
      if (!ev.dataTransfer) return;
      // ドロップされてきたものがファイルだったら
      if (ev.dataTransfer.files.length > 0) {
        os.launchUploader(Array.from(ev.dataTransfer.files), { folderId: folder.value?.id ?? null });
        return;
      }
      //#region ドライブのファイル
      {
        const droppedData = getDragData(ev, "driveFiles");
        if (droppedData != null) {
          misskeyApi("drive/files/move-bulk", {
            fileIds: droppedData.map((f) => f.id),
            folderId: folder.value ? folder.value.id : null
          }).then(() => {
            globalEvents.emit("driveFilesUpdated", droppedData.map((x) => ({
              ...x,
              folderId: folder.value ? folder.value.id : null,
              folder: folder.value
            })));
          });
        }
      }
      //#endregion
      //#region ドライブのフォルダ
      {
        const droppedData = getDragData(ev, "driveFolders");
        if (droppedData != null) {
          const droppedFolder = droppedData[0];
          // 移動先が自分自身ならreject
          if (folder.value && droppedFolder.id === folder.value.id) return false;
          if (foldersPaginator.items.value.some((f) => f.id === droppedFolder.id)) return false;
          misskeyApi("drive/folders/update", {
            folderId: droppedFolder.id,
            parentId: folder.value ? folder.value.id : null
          }).then(() => {
            globalEvents.emit("driveFoldersUpdated", [droppedFolder].map((x) => ({
              ...x,
              parentId: folder.value ? folder.value.id : null,
              parent: folder.value
            })));
          }).catch((err) => {
            switch (err.code) {
              case "RECURSIVE_NESTING":
                claimAchievement("driveFolderCircularReference");
                os.alert({
                  type: "error",
                  title: i18n.ts.unableToProcess,
                  text: i18n.ts.circularReferenceFolder
                });
                break;
              default: os.alert({
                type: "error",
                text: i18n.ts.somethingHappened
              });
            }
          });
        }
      }
      //#endregion
    }
    function onUploadRequested(files, folder) {
      os.launchUploader(files, { folderId: folder?.id ?? null });
    }
    async function urlUpload() {
      const { canceled, result: url } = await os.inputText({
        title: i18n.ts.uploadFromUrl,
        type: "url",
        placeholder: i18n.ts.uploadFromUrlDescription
      });
      if (canceled || !url) return;
      await os.apiWithDialog("drive/files/upload-from-url", {
        url,
        folderId: folder.value ? folder.value.id : undefined
      });
      os.alert({
        title: i18n.ts.uploadFromUrlRequested,
        text: i18n.ts.uploadFromUrlMayTakeTime
      });
    }
    async function createFolder() {
      const { canceled, result: name } = await os.inputText({
        title: i18n.ts.createFolder,
        placeholder: i18n.ts.folderName
      });
      if (canceled || name == null) return;
      const createdFolder = await os.apiWithDialog("drive/folders/create", {
        name,
        parentId: folder.value ? folder.value.id : undefined
      });
      foldersPaginator.prepend(createdFolder);
    }
    async function renameFolder(folderToRename) {
      const { canceled, result: name } = await os.inputText({
        title: i18n.ts.renameFolder,
        placeholder: i18n.ts.inputNewFolderName,
        default: folderToRename.name
      });
      if (canceled) return;
      const updatedFolder = await os.apiWithDialog("drive/folders/update", {
        folderId: folderToRename.id,
        name
      });
      globalEvents.emit("driveFoldersUpdated", [updatedFolder]);
    }
    function deleteFolder(folderToDelete) {
      misskeyApi("drive/folders/delete", { folderId: folderToDelete.id }).then(() => {
        // 削除時に親フォルダに移動
        cd(folderToDelete.parentId);
        globalEvents.emit("driveFoldersDeleted", [folderToDelete]);
      }).catch((err) => {
        switch (err.id) {
          case "b0fc8a17-963c-405d-bfbc-859a487295e1":
            os.alert({
              type: "error",
              title: i18n.ts.unableToDelete,
              text: i18n.ts.hasChildFilesOrFolders
            });
            break;
          default: os.alert({
            type: "error",
            text: i18n.ts.unableToDelete
          });
        }
      });
    }
    function onFileClick(ev, file) {
      if (ev.shiftKey) {
        isEditMode.value = true;
      }
      if (props.select === "file" || isEditMode.value) {
        const isAlreadySelected = selectedFiles.value.some((f) => f.id === file.id);
        if (isEditMode.value) {
          if (isAlreadySelected) {
            selectedFiles.value = selectedFiles.value.filter((f) => f.id !== file.id);
          } else {
            selectedFiles.value.push(file);
          }
          return;
        }
        if (props.multiple) {
          if (isAlreadySelected) {
            selectedFiles.value = selectedFiles.value.filter((f) => f.id !== file.id);
          } else {
            selectedFiles.value.push(file);
          }
        } else {
          if (isAlreadySelected) {} else {
            selectedFiles.value = [file];
          }
        }
      } else {
        os.popupMenu(getDriveFileMenu(file, folder.value), ev.currentTarget ?? ev.target ?? undefined);
      }
    }
    function chooseFolder(folderToChoose) {
      const isAlreadySelected = selectedFolders.value.some((f) => f.id === folderToChoose.id);
      if (props.multiple) {
        if (isAlreadySelected) {
          selectedFolders.value = selectedFolders.value.filter((f) => f.id !== folderToChoose.id);
        } else {
          selectedFolders.value.push(folderToChoose);
        }
      } else {
        if (isAlreadySelected) {} else {
          selectedFolders.value = [folderToChoose];
        }
      }
    }
    function unchoseFolder(folderToUnchose) {
      selectedFolders.value = selectedFolders.value.filter((f) => f.id !== folderToUnchose.id);
    }
    function cd(target) {
      if (!target) {
        goRoot();
        return;
      } else if (typeof target === "object") {
        target = target.id;
      }
      fetching.value = true;
      misskeyApi("drive/folders/show", { folderId: target }).then((folderToMove) => {
        folder.value = folderToMove;
        hierarchyFolders.value = [];
        const dive = (folderToDive) => {
          hierarchyFolders.value.unshift(folderToDive);
          if (folderToDive.parent) dive(folderToDive.parent);
        };
        if (folderToMove.parent) dive(folderToMove.parent);
        initialize();
      });
    }
    async function moveFilesBulk() {
      if (selectedFiles.value.length === 0) return;
      const { canceled, folders } = await selectDriveFolder(folder.value ? folder.value.id : null);
      if (canceled) return;
      await os.apiWithDialog("drive/files/move-bulk", {
        fileIds: selectedFiles.value.map((f) => f.id),
        folderId: folders[0] ? folders[0].id : null
      });
      globalEvents.emit("driveFilesUpdated", selectedFiles.value.map((x) => ({
        ...x,
        folderId: folders[0] ? folders[0].id : null,
        folder: folders[0] ?? null
      })));
    }
    function goRoot() {
      // 既にrootにいるなら何もしない
      if (folder.value == null) return;
      folder.value = null;
      hierarchyFolders.value = [];
      initialize();
    }
    function getMenu() {
      const menu = [];
      menu.push({
        text: i18n.ts.addFile,
        type: "label"
      }, {
        text: i18n.ts.upload,
        icon: "ti ti-upload",
        action: () => {
          chooseFileFromPcAndUpload({
            multiple: true,
            folderId: folder.value?.id
          });
        }
      }, {
        text: i18n.ts.fromUrl,
        icon: "ti ti-link",
        action: () => {
          urlUpload();
        }
      }, { type: "divider" }, {
        text: folder.value ? folder.value.name : i18n.ts.drive,
        type: "label"
      });
      menu.push({
        type: "parent",
        text: i18n.ts.sort,
        icon: "ti ti-arrows-sort",
        children: [
          {
            text: `${i18n.ts.registeredDate} (${i18n.ts.descendingOrder})`,
            icon: "ti ti-sort-descending-letters",
            action: () => {
              sortModeSelect.value = "+createdAt";
            },
            active: sortModeSelect.value === "+createdAt"
          },
          {
            text: `${i18n.ts.registeredDate} (${i18n.ts.ascendingOrder})`,
            icon: "ti ti-sort-ascending-letters",
            action: () => {
              sortModeSelect.value = "-createdAt";
            },
            active: sortModeSelect.value === "-createdAt"
          },
          {
            text: `${i18n.ts.size} (${i18n.ts.descendingOrder})`,
            icon: "ti ti-sort-descending-letters",
            action: () => {
              sortModeSelect.value = "+size";
            },
            active: sortModeSelect.value === "+size"
          },
          {
            text: `${i18n.ts.size} (${i18n.ts.ascendingOrder})`,
            icon: "ti ti-sort-ascending-letters",
            action: () => {
              sortModeSelect.value = "-size";
            },
            active: sortModeSelect.value === "-size"
          },
          {
            text: `${i18n.ts.name} (${i18n.ts.descendingOrder})`,
            icon: "ti ti-sort-descending-letters",
            action: () => {
              sortModeSelect.value = "+name";
            },
            active: sortModeSelect.value === "+name"
          },
          {
            text: `${i18n.ts.name} (${i18n.ts.ascendingOrder})`,
            icon: "ti ti-sort-ascending-letters",
            action: () => {
              sortModeSelect.value = "-name";
            },
            active: sortModeSelect.value === "-name"
          }
        ]
      });
      if (folder.value) {
        menu.push({
          text: i18n.ts.renameFolder,
          icon: "ti ti-forms",
          action: () => {
            if (folder.value) renameFolder(folder.value);
          }
        }, {
          text: i18n.ts.deleteFolder,
          icon: "ti ti-trash",
          action: () => {
            deleteFolder(folder.value);
          }
        });
      }
      menu.push({
        text: i18n.ts.createFolder,
        icon: "ti ti-folder-plus",
        action: () => {
          createFolder();
        }
      }, { type: "divider" }, {
        type: "switch",
        text: i18n.ts.edit,
        icon: "ti ti-pointer",
        ref: isEditMode
      });
      return menu;
    }
    function showMenu(ev) {
      os.popupMenu(getMenu(), ev.currentTarget ?? ev.target ?? undefined);
    }
    function onContextmenu(ev) {
      os.contextMenu(getMenu(), ev);
    }
    useGlobalEvent("driveFileCreated", (file) => {
      if (file.folderId === (folder.value?.id ?? null)) {
        filesPaginator.prepend(file);
      }
    });
    useGlobalEvent("driveFilesUpdated", (files) => {
      for (const f of files) {
        if (filesPaginator.items.value.some((x) => x.id === f.id)) {
          if (f.folderId === (folder.value?.id ?? null)) {
            filesPaginator.updateItem(f.id, () => f);
          } else {
            filesPaginator.removeItem(f.id);
          }
        } else {
          if (f.folderId === (folder.value?.id ?? null)) {
            filesPaginator.prepend(f);
          }
        }
      }
    });
    useGlobalEvent("driveFilesDeleted", (files) => {
      for (const f of files) {
        filesPaginator.removeItem(f.id);
      }
    });
    useGlobalEvent("driveFoldersUpdated", (folders) => {
      for (const f of folders) {
        if (foldersPaginator.items.value.some((x) => x.id === f.id)) {
          if (f.parentId === (folder.value?.id ?? null)) {
            foldersPaginator.updateItem(f.id, () => f);
          } else {
            foldersPaginator.removeItem(f.id);
          }
        } else {
          if (f.parentId === (folder.value?.id ?? null)) {
            foldersPaginator.prepend(f);
          }
        }
      }
    });
    useGlobalEvent("driveFoldersDeleted", (folders) => {
      for (const f of folders) {
        foldersPaginator.removeItem(f.id);
      }
    });
    let connection = null;
    onMounted(() => {
      if (store.s.realtimeMode) {
        connection = useStream().useChannel("drive");
        connection.on("fileCreated", onStreamDriveFileCreated);
      }
      if (props.initialFolder) {
        cd(props.initialFolder);
      } else {
        initialize();
      }
    });
    onActivated(() => {});
    onBeforeUnmount(() => {
      if (connection != null) {
        connection.dispose();
      }
    });
    return (_ctx, _cache) => {
      const _component_MkTip = _resolveComponent("MkTip");
      const _component_MkStickyContainer = _resolveComponent("MkStickyContainer");
      const _component_MkLoading = _resolveComponent("MkLoading");
      const _directive_anim = _resolveDirective("anim");
      const _directive_appear = _resolveDirective("appear");
      return _openBlock(), _createBlock(_component_MkStickyContainer, { style: "background: var(--MI_THEME-bg);" }, {
        header: _withCtx(() => [_createElementVNode(
          "nav",
          { class: _normalizeClass(_ctx.$style.nav) },
          [_createElementVNode(
            "div",
            {
              class: _normalizeClass(_ctx.$style.navPath),
              onContextmenu: _cache[0] || (_cache[0] = _withModifiers(() => {}, ["prevent", "stop"]))
            },
            [
              _createVNode(XNavFolder, {
                class: _normalizeClass([_ctx.$style.navPathItem, { [_ctx.$style.navCurrent]: folder.value == null }]),
                parentFolder: folder.value,
                onClick: _cache[1] || (_cache[1] = ($event) => cd(null)),
                onUpload: onUploadRequested
              }, null, 10, ["parentFolder"]),
              (_openBlock(true), _createElementBlock(
                _Fragment,
                null,
                _renderList(hierarchyFolders.value, (f) => {
                  return _openBlock(), _createElementBlock(
                    _Fragment,
                    null,
                    [_createElementVNode(
                      "span",
                      { class: _normalizeClass([_ctx.$style.navPathItem, _ctx.$style.navSeparator]) },
                      [_hoisted_1],
                      2
                      /* CLASS */
                    ), _createVNode(XNavFolder, {
                      folder: f,
                      parentFolder: folder.value,
                      class: _normalizeClass([_ctx.$style.navPathItem]),
                      onClick: ($event) => cd(f),
                      onUpload: onUploadRequested
                    }, null, 10, [
                      "folder",
                      "parentFolder",
                      "onClick"
                    ])],
                    64
                    /* STABLE_FRAGMENT */
                  );
                }),
                256
                /* UNKEYED_FRAGMENT */
              )),
              folder.value != null ? (_openBlock(), _createElementBlock(
                "span",
                {
                  key: 0,
                  class: _normalizeClass([_ctx.$style.navPathItem, _ctx.$style.navSeparator])
                },
                [_hoisted_2],
                2
                /* CLASS */
              )) : _createCommentVNode("v-if", true),
              folder.value != null ? (_openBlock(), _createElementBlock(
                "span",
                {
                  key: 0,
                  class: _normalizeClass([_ctx.$style.navPathItem, _ctx.$style.navCurrent])
                },
                _toDisplayString(folder.value.name),
                3
                /* TEXT, CLASS */
              )) : _createCommentVNode("v-if", true)
            ],
            34
            /* CLASS, NEED_HYDRATION */
          ), _createElementVNode(
            "button",
            {
              class: _normalizeClass(["_button", _ctx.$style.navMenu]),
              onClick: showMenu
            },
            [_hoisted_3],
            2
            /* CLASS */
          )],
          2
          /* CLASS */
        )]),
        footer: _withCtx(() => [isEditMode.value ? (_openBlock(), _createElementBlock(
          "div",
          {
            key: 0,
            class: _normalizeClass(_ctx.$style.footer)
          },
          [_createVNode(MkButton, {
            primary: "",
            rounded: "",
            onClick: _cache[2] || (_cache[2] = ($event) => moveFilesBulk())
          }, {
            default: _withCtx(() => [
              _hoisted_9,
              _createTextVNode(" "),
              _createTextVNode(
                _toDisplayString(_unref(i18n).ts.move),
                1
                /* TEXT */
              ),
              _createTextVNode("...")
            ]),
            _: 1
          })],
          2
          /* CLASS */
        )) : _createCommentVNode("v-if", true)]),
        default: _withCtx(() => [_createElementVNode("div", null, [
          __props.select === "folder" ? (_openBlock(), _createElementBlock("div", { key: 0 }, [folder.value == null ? (_openBlock(), _createElementBlock(
            _Fragment,
            { key: 0 },
            [!isRootSelected.value ? (_openBlock(), _createBlock(MkButton, {
              key: 0,
              onClick: _cache[3] || (_cache[3] = ($event) => isRootSelected.value = true)
            }, {
              default: _withCtx(() => [
                _hoisted_4,
                _createTextVNode(" "),
                _createTextVNode(
                  _toDisplayString(_unref(i18n).ts.selectFolder),
                  1
                  /* TEXT */
                )
              ]),
              _: 1
            })) : (_openBlock(), _createBlock(MkButton, {
              key: 1,
              onClick: _cache[4] || (_cache[4] = ($event) => isRootSelected.value = false)
            }, {
              default: _withCtx(() => [
                _hoisted_5,
                _createTextVNode(" "),
                _createTextVNode(
                  _toDisplayString(_unref(i18n).ts.unselectFolder),
                  1
                  /* TEXT */
                )
              ]),
              _: 1
            }))],
            64
            /* STABLE_FRAGMENT */
          )) : (_openBlock(), _createElementBlock(
            _Fragment,
            { key: 1 },
            [!selectedFolders.value.some((f) => f.id === folder.value.id) ? (_openBlock(), _createBlock(MkButton, {
              key: 0,
              onClick: _cache[5] || (_cache[5] = ($event) => selectedFolders.value.push(folder.value))
            }, {
              default: _withCtx(() => [
                _hoisted_6,
                _createTextVNode(" "),
                _createTextVNode(
                  _toDisplayString(_unref(i18n).ts.selectFolder),
                  1
                  /* TEXT */
                )
              ]),
              _: 1
            })) : (_openBlock(), _createBlock(MkButton, {
              key: 1,
              onClick: _cache[6] || (_cache[6] = ($event) => selectedFolders.value = selectedFolders.value.filter((f) => f.id !== folder.value.id))
            }, {
              default: _withCtx(() => [
                _hoisted_7,
                _createTextVNode(" "),
                _createTextVNode(
                  _toDisplayString(_unref(i18n).ts.unselectFolder),
                  1
                  /* TEXT */
                )
              ]),
              _: 1
            }))],
            64
            /* STABLE_FRAGMENT */
          ))])) : _createCommentVNode("v-if", true),
          _createElementVNode(
            "div",
            {
              ref: "main",
              class: _normalizeClass([_ctx.$style.main, { [_ctx.$style.fetching]: fetching.value }]),
              onDragover: _withModifiers(onDragover, ["prevent", "stop"]),
              onDragenter,
              onDragleave,
              onDrop: _withModifiers(onDrop, ["prevent", "stop"]),
              onContextmenu: _withModifiers(onContextmenu, ["stop"])
            },
            [
              _createElementVNode(
                "div",
                { class: _normalizeClass(_ctx.$style.tipContainer) },
                [_createVNode(_component_MkTip, { k: "drive" }, {
                  default: _withCtx(() => [_createElementVNode("div", { innerHTML: _unref(i18n).ts.driveAboutTip }, null, 8, ["innerHTML"])]),
                  _: 1
                })],
                2
                /* CLASS */
              ),
              _createElementVNode(
                "div",
                { class: _normalizeClass(_ctx.$style.folders) },
                [(_openBlock(true), _createElementBlock(
                  _Fragment,
                  null,
                  _renderList(_unref(foldersPaginator).items.value, (f, i) => {
                    return _withDirectives((_openBlock(), _createBlock(XFolder, {
                      key: f.id,
                      folder: f,
                      selectMode: __props.select === "folder",
                      isSelected: selectedFolders.value.some((x) => x.id === f.id),
                      onChosen: chooseFolder,
                      onUnchose: unchoseFolder,
                      onClick: ($event) => cd(f),
                      onUpload: onUploadRequested,
                      onDragstart: ($event) => isDragSource.value = true,
                      onDragend: ($event) => isDragSource.value = false
                    }, null, 8, [
                      "folder",
                      "selectMode",
                      "isSelected",
                      "onClick",
                      "onDragstart",
                      "onDragend"
                    ])), [[_directive_anim, i]]);
                  }),
                  128
                  /* KEYED_FRAGMENT */
                ))],
                2
                /* CLASS */
              ),
              _unref(foldersPaginator).canFetchOlder.value ? (_openBlock(), _createBlock(MkButton, {
                key: 0,
                primary: "",
                rounded: "",
                onClick: _cache[7] || (_cache[7] = ($event) => _unref(foldersPaginator).fetchOlder())
              }, {
                default: _withCtx(() => [_createTextVNode(
                  _toDisplayString(_unref(i18n).ts.loadMore),
                  1
                  /* TEXT */
                )]),
                _: 1
              })) : _createCommentVNode("v-if", true),
              shouldBeGroupedByDate.value ? (_openBlock(), _createElementBlock(
                _Fragment,
                { key: 0 },
                [(_openBlock(true), _createElementBlock(
                  _Fragment,
                  null,
                  _renderList(_unref(filesTimeline), (item, i) => {
                    return _openBlock(), _createBlock(
                      _component_MkStickyContainer,
                      { key: `${item.date.getFullYear()}/${item.date.getMonth() + 1}` },
                      {
                        header: _withCtx(() => [_createElementVNode(
                          "div",
                          { class: _normalizeClass(_ctx.$style.date) },
                          [_createElementVNode("span", null, [_hoisted_8, _createTextVNode(
                            " " + _toDisplayString(item.date.getFullYear()) + "/" + _toDisplayString(item.date.getMonth() + 1),
                            1
                            /* TEXT */
                          )])],
                          2
                          /* CLASS */
                        )]),
                        default: _withCtx(() => [_createVNode(_TransitionGroup, {
                          tag: "div",
                          enterActiveClass: _unref(prefer).s.animation ? _ctx.$style.transition_files_enterActive : "",
                          leaveActiveClass: _unref(prefer).s.animation ? _ctx.$style.transition_files_leaveActive : "",
                          enterFromClass: _unref(prefer).s.animation ? _ctx.$style.transition_files_enterFrom : "",
                          leaveToClass: _unref(prefer).s.animation ? _ctx.$style.transition_files_leaveTo : "",
                          moveClass: _unref(prefer).s.animation ? _ctx.$style.transition_files_move : "",
                          class: _normalizeClass(_ctx.$style.files)
                        }, {
                          default: _withCtx(() => [(_openBlock(true), _createElementBlock(
                            _Fragment,
                            null,
                            _renderList(item.items, (file) => {
                              return _openBlock(), _createBlock(XFile, {
                                key: file.id,
                                file,
                                folder: folder.value,
                                isSelected: selectedFiles.value.some((x) => x.id === file.id),
                                onClick: ($event) => onFileClick($event, file),
                                onDragstart: ($event) => onFileDragstart(file, $event),
                                onDragend: ($event) => isDragSource.value = false
                              }, null, 8, [
                                "file",
                                "folder",
                                "isSelected",
                                "onClick",
                                "onDragstart",
                                "onDragend"
                              ]);
                            }),
                            128
                            /* KEYED_FRAGMENT */
                          ))]),
                          _: 2
                        }, 1034, [
                          "enterActiveClass",
                          "leaveActiveClass",
                          "enterFromClass",
                          "leaveToClass",
                          "moveClass"
                        ])]),
                        _: 2
                      },
                      1024
                      /* DYNAMIC_SLOTS */
                    );
                  }),
                  128
                  /* KEYED_FRAGMENT */
                ))],
                64
                /* STABLE_FRAGMENT */
              )) : (_openBlock(), _createBlock(_TransitionGroup, {
                key: 1,
                tag: "div",
                enterActiveClass: _unref(prefer).s.animation ? _ctx.$style.transition_files_enterActive : "",
                leaveActiveClass: _unref(prefer).s.animation ? _ctx.$style.transition_files_leaveActive : "",
                enterFromClass: _unref(prefer).s.animation ? _ctx.$style.transition_files_enterFrom : "",
                leaveToClass: _unref(prefer).s.animation ? _ctx.$style.transition_files_leaveTo : "",
                moveClass: _unref(prefer).s.animation ? _ctx.$style.transition_files_move : "",
                class: _normalizeClass(_ctx.$style.files)
              }, {
                default: _withCtx(() => [(_openBlock(true), _createElementBlock(
                  _Fragment,
                  null,
                  _renderList(_unref(filesPaginator).items.value, (file) => {
                    return _openBlock(), _createBlock(XFile, {
                      key: file.id,
                      file,
                      folder: folder.value,
                      isSelected: selectedFiles.value.some((x) => x.id === file.id),
                      onClick: ($event) => onFileClick($event, file),
                      onDragstart: ($event) => onFileDragstart(file, $event),
                      onDragend: ($event) => isDragSource.value = false
                    }, null, 8, [
                      "file",
                      "folder",
                      "isSelected",
                      "onClick",
                      "onDragstart",
                      "onDragend"
                    ]);
                  }),
                  128
                  /* KEYED_FRAGMENT */
                ))]),
                _: 2
              }, 1034, [
                "enterActiveClass",
                "leaveActiveClass",
                "enterFromClass",
                "leaveToClass",
                "moveClass"
              ])),
              _withDirectives(_createVNode(
                MkButton,
                {
                  class: _normalizeClass(_ctx.$style.loadMore),
                  primary: "",
                  rounded: "",
                  onClick: fetchMoreFiles
                },
                {
                  default: _withCtx(() => [_createTextVNode(
                    _toDisplayString(_unref(i18n).ts.loadMore),
                    1
                    /* TEXT */
                  )]),
                  _: 1
                },
                2
                /* CLASS */
              ), [[_directive_appear, shouldEnableInfiniteScroll.value ? fetchMoreFiles : null], [_vShow, canFetchFiles.value]]),
              _unref(filesPaginator).items.value.length == 0 && _unref(foldersPaginator).items.value.length == 0 && !fetching.value ? (_openBlock(), _createElementBlock(
                "div",
                {
                  key: 0,
                  class: _normalizeClass(_ctx.$style.empty)
                },
                [
                  draghover.value ? (_openBlock(), _createElementBlock(
                    "div",
                    { key: 0 },
                    _toDisplayString(_unref(i18n).ts.dropHereToUpload),
                    1
                    /* TEXT */
                  )) : _createCommentVNode("v-if", true),
                  !draghover.value && folder.value == null ? (_openBlock(), _createElementBlock("div", { key: 0 }, [_createElementVNode(
                    "strong",
                    null,
                    _toDisplayString(_unref(i18n).ts.emptyDrive),
                    1
                    /* TEXT */
                  )])) : _createCommentVNode("v-if", true),
                  !draghover.value && folder.value != null ? (_openBlock(), _createElementBlock(
                    "div",
                    { key: 0 },
                    _toDisplayString(_unref(i18n).ts.emptyFolder),
                    1
                    /* TEXT */
                  )) : _createCommentVNode("v-if", true)
                ],
                2
                /* CLASS */
              )) : _createCommentVNode("v-if", true)
            ],
            34
            /* CLASS, NEED_HYDRATION */
          ),
          fetching.value ? (_openBlock(), _createBlock(_component_MkLoading, { key: 0 })) : _createCommentVNode("v-if", true),
          draghover.value ? (_openBlock(), _createElementBlock(
            "div",
            {
              key: 0,
              class: _normalizeClass(_ctx.$style.dropzone)
            },
            null,
            2
            /* CLASS */
          )) : _createCommentVNode("v-if", true)
        ])]),
        _: 1
      });
    };
  }
};
