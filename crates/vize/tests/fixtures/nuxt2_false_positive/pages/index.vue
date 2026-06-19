<script lang="ts">
import { defineComponent, type PropType } from "vue";
import { useContext } from "@nuxtjs/composition-api";
import KeyboardPanel, { type Folder } from "~/components/KeyboardPanel.vue";
import OverlayPanel from "~/components/OverlayPanel.vue";

const componentProps = {
  folders: {
    type: Array as PropType<Folder[]>,
    required: true,
  },
};

function usePanelState() {
  return {
    panelTitle: "Folders",
    selectedId: "root",
  };
}

export default defineComponent({
  components: { KeyboardPanel, OverlayPanel },
  props: componentProps,
  setup(props, { emit }) {
    const context = useContext();
    const repoItem = context.$accountRepository.find(props.folders[0].id);
    context.$logger.info(context.$auth.userName());
    context.$gtm.push({ name: repoItem.label });

    const onClickFolder = (folder: Folder) => {
      emit("click-folder", folder);
      context.$logger.info(folder.label);
    };
    const onInputMathKey = (key: string) => {
      emit("input:math-key", key);
    };
    const onUpdateIsOpenedOverlayLoading = (value: boolean) => {
      emit("update:is-opened-overlay-loading", value);
    };

    return {
      ...usePanelState(),
      repoLabel: repoItem.label,
      onClickFolder,
      onInputMathKey,
      onUpdateIsOpenedOverlayLoading,
    };
  },
});
</script>

<template>
  <section>
    <h1>{{ panelTitle }} {{ selectedId }} {{ repoLabel }}</h1>
    <KeyboardPanel
      :folders="folders"
      @click-folder="onClickFolder"
      @input:math-key="onInputMathKey"
    />
    <OverlayPanel @update:is-opened-overlay-loading="onUpdateIsOpenedOverlayLoading" />
  </section>
</template>
