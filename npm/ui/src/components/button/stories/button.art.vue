<script setup lang="ts">
import { Button } from "../index.ts";
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
  <variant name="Variants and tones">
    <Button type="button" variant="solid" tone="accent">Create release</Button>
    <Button type="button" variant="outline" tone="neutral">Preview</Button>
    <Button type="button" variant="quiet" tone="danger">Delete</Button>
  </variant>
  <variant name="Sizes">
    <Button type="button" size="sm">Small</Button>
    <Button type="button" size="md">Medium</Button>
    <Button type="button" size="lg">Large</Button>
  </variant>
  <variant name="Submit">
    <form>
      <Button type="submit">Send</Button>
    </form>
  </variant>
</art>
