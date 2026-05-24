<script setup lang="ts">
import type {} from "./shims";
import { computed, reactive, ref } from "vue";
import {
  DialogClose,
  DialogContent,
  DialogOverlay,
  DialogPortal,
  DialogRoot,
  DialogTitle,
  DialogTrigger,
} from "reka-ui";
import PrimeButton from "primevue/button";
import InputText from "primevue/inputtext";
import { Alert as AAlert, Button as AButton, Select as ASelect, Space as ASpace } from "ant-design-vue";
import UButton from "@nuxt/ui/components/Button.vue";
import UCard from "@nuxt/ui/components/Card.vue";
import UInput from "@nuxt/ui/components/Input.vue";
import { QBtn, QInput, QSelect } from "quasar";
import {
  ElButton,
  ElForm,
  ElFormItem,
  ElInput,
  ElOption,
  ElSelect,
} from "element-plus";
import { IonButton, IonInput, IonItem, IonList } from "@ionic/vue";
import { Button as VanButton, Cell as VanCell, Field as VanField } from "vant";
import { NButton, NCard, NSelect } from "naive-ui";
import { FormKit } from "@formkit/vue";
import type { ProductOption } from "./types";

const dialogOpen = ref(false);
const primeValue = ref("PrimeVue");
const antValue = ref("ant-design-vue");
const nuxtValue = ref("Nuxt UI");
const quasarValue = ref("Quasar");
const quasarSelectValue = ref("quasar");
const elementValue = ref("element-plus");
const ionicValue = ref("Ionic Vue");
const vantValue = ref("Vant");
const naiveValue = ref("naive-ui");
const formkitValue = ref("FormKit");

const elementModel = reactive({
  name: "Element Plus",
  framework: "element-plus",
});

const libraryOptions = [
  { label: "Ant Design Vue", value: "ant-design-vue" },
  { label: "Element Plus", value: "element-plus" },
  { label: "Naive UI", value: "naive-ui" },
  { label: "Quasar", value: "quasar" },
] satisfies ProductOption[];

const activeLabel = computed(() => {
  return libraryOptions.find((option) => option.value === antValue.value)?.label ?? "Unknown";
});

function promoteLibrary(value: string): void {
  naiveValue.value = value;
  elementModel.framework = value;
}
</script>

<template>
  <section class="ecosystem-grid" aria-label="Vue UI library coverage">
    <DialogRoot v-model:open="dialogOpen">
      <DialogTrigger as-child>
        <PrimeButton label="Open Reka dialog" severity="secondary" />
      </DialogTrigger>
      <DialogPortal>
        <DialogOverlay class="overlay" />
        <DialogContent class="dialog">
          <DialogTitle>Reka UI dialog for {{ activeLabel }}</DialogTitle>
          <p>Dialog state: {{ dialogOpen ? "open" : "closed" }}</p>
          <DialogClose as-child>
            <AButton type="primary">Close with Ant Design Vue</AButton>
          </DialogClose>
        </DialogContent>
      </DialogPortal>
    </DialogRoot>

    <ASpace direction="vertical" class="library-panel">
      <AAlert type="success" show-icon :message="`Ant Design Vue selected: ${activeLabel}`" />
      <ASelect v-model:value="antValue" :options="libraryOptions" style="width: 240px" />
      <AButton type="primary" @click="promoteLibrary(antValue)">Promote</AButton>
    </ASpace>

    <UCard class="library-panel">
      <template #header>Nuxt UI</template>
      <UInput v-model="nuxtValue" placeholder="Nuxt UI input" />
      <UButton color="primary" variant="solid">Use {{ nuxtValue }}</UButton>
    </UCard>

    <div class="library-panel">
      <InputText v-model="primeValue" aria-label="PrimeVue input" />
      <PrimeButton :label="primeValue" severity="contrast" />
    </div>

    <div class="library-panel">
      <QInput v-model="quasarValue" label="Quasar input" dense outlined />
      <QSelect
        v-model="quasarSelectValue"
        :options="libraryOptions"
        emit-value
        map-options
        dense
        outlined
        label="Quasar select"
      />
      <QBtn color="primary" :label="quasarValue" />
    </div>

    <ElForm class="library-panel" :model="elementModel" label-width="96px">
      <ElFormItem label="Name">
        <ElInput v-model="elementModel.name" />
      </ElFormItem>
      <ElFormItem label="Library">
        <ElSelect v-model="elementModel.framework" placeholder="Pick one">
          <ElOption
            v-for="option in libraryOptions"
            :key="option.value"
            :label="option.label"
            :value="option.value"
          />
        </ElSelect>
      </ElFormItem>
      <ElButton type="primary" @click="elementValue = elementModel.framework">
        Element Plus: {{ elementValue }}
      </ElButton>
    </ElForm>

    <IonList class="library-panel">
      <IonItem>
        <IonInput v-model="ionicValue" label="Ionic Vue" label-placement="stacked" />
      </IonItem>
      <IonItem>
        <IonButton :strong="true">{{ ionicValue }}</IonButton>
      </IonItem>
    </IonList>

    <div class="library-panel">
      <VanField v-model="vantValue" label="Vant" placeholder="Mobile component value" />
      <VanCell title="Vant cell" :value="vantValue" />
      <VanButton type="primary" size="small">Confirm Vant</VanButton>
    </div>

    <NCard class="library-panel" title="Naive UI" size="small">
      <NSelect v-model:value="naiveValue" :options="libraryOptions" />
      <NButton type="primary" @click="promoteLibrary(naiveValue)">Apply Naive UI</NButton>
    </NCard>

    <div class="library-panel">
      <FormKit
        v-model="formkitValue"
        type="text"
        name="framework"
        label="FormKit"
        validation="required"
      />
      <p>FormKit value: {{ formkitValue }}</p>
    </div>
  </section>
</template>

<style scoped>
.ecosystem-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
  gap: 12px;
}

.library-panel {
  border: 1px solid #d0d7de;
  border-radius: 8px;
  padding: 12px;
}

.overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.3);
}

.dialog {
  position: fixed;
  top: 20%;
  left: 50%;
  width: min(420px, 90vw);
  transform: translateX(-50%);
  border-radius: 8px;
  background: white;
  padding: 16px;
}
</style>
