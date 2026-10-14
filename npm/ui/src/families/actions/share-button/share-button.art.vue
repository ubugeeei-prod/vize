<script setup lang="ts">
import { ShareButton } from "./share-button.ts";
</script>

<art title="Share Button" status="ready">
  <variant name="Default" default>
    <ShareButton title="Vize documentation" url="https://vizejs.dev" />
  </variant>
  <variant name="Text and custom label">
    <ShareButton
      title="Vize documentation"
      text="Read the Vize docs"
      url="https://vizejs.dev"
      idle-label="Share docs"
    />
  </variant>
  <variant name="Disabled">
    <ShareButton title="Vize documentation" url="https://vizejs.dev" disabled />
  </variant>
</art>
