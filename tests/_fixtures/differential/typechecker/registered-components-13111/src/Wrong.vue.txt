<script lang="ts">
import { defineComponent } from "vue";
import { Menu } from "menu-library";
import Child from "./Child.vue";
import * as ui from "./ui";

export default defineComponent({
  components: { Aliased: Child, AliasedMenu: Menu, MemberButton: ui.Button },
});
</script>

<template>
  <Aliased :label="1" />
  <AliasedMenu :as="2" />
  <MemberButton :size="3" />
  <NotRegistered />
</template>
