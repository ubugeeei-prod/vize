<script setup lang="ts">
import { Button } from "./button.ts";
</script>

<art title="Button" status="ready">
  <variant name="Default" default>
    <Button type="button">Save draft</Button>
  </variant>
  <variant name="Loading">
    <Button v-slot="{ loading }" type="button" loading>
      {{ loading ? "Uploading…" : "Upload" }}
    </Button>
  </variant>
  <variant name="Disabled">
    <Button type="button" disabled>Publish</Button>
  </variant>
  <variant name="Submit">
    <form>
      <Button type="submit">Send</Button>
    </form>
  </variant>
</art>
