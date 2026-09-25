<!-- Cascader that picks a delivery region by drilling from country to prefecture to city. -->
<script setup lang="ts">
import { ref, useId } from "vue";

import {
  Cascader,
  CascaderColumn,
  CascaderContent,
  CascaderItem,
  CascaderTrigger,
  CascaderValue,
} from "../cascader.ts";

interface Region {
  readonly value: string;
  readonly label: string;
  readonly children?: readonly Region[];
}

const regions: readonly Region[] = [
  {
    value: "jp",
    label: "Japan",
    children: [
      {
        value: "tokyo",
        label: "Tokyo",
        children: [
          { value: "shibuya", label: "Shibuya" },
          { value: "shinjuku", label: "Shinjuku" },
        ],
      },
      { value: "osaka", label: "Osaka", children: [{ value: "kita", label: "Kita" }] },
    ],
  },
  {
    value: "fr",
    label: "France",
    children: [
      { value: "paris", label: "Paris" },
      { value: "lyon", label: "Lyon" },
    ],
  },
];
const path = ref<readonly Region[]>([]);
const labelId = useId();
</script>

<template>
  <div>
    <span :id="labelId">Delivery region</span>
    <Cascader
      v-slot="{ columns }"
      v-model="path"
      :options="regions"
      by="value"
      :item-text="(region: Region) => region.label"
      placeholder="Choose a region"
      name="region"
    >
      <CascaderTrigger :aria-labelledby="labelId">
        <CascaderValue />
      </CascaderTrigger>
      <CascaderContent>
        <CascaderColumn v-for="column in columns" :key="column.level" :level="column.level">
          <CascaderItem v-for="region in column.options" :key="region.value" :value="region">
            {{ region.label }}
          </CascaderItem>
        </CascaderColumn>
      </CascaderContent>
    </Cascader>
  </div>
</template>
