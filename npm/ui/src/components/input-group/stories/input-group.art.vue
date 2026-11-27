<script setup lang="ts">
import {
  InputGroup,
  InputGroupAddon,
  InputGroupButton,
  InputGroupInput,
  InputGroupTextarea,
} from "../index.ts";
</script>

<art title="Input Group" status="ready">
  <variant name="Default" default>
    <InputGroup label="Website">
      <InputGroupAddon>https://</InputGroupAddon>
      <InputGroupInput model-value="vize.dev" type="url" />
      <InputGroupButton>Copy</InputGroupButton>
    </InputGroup>
  </variant>
  <variant name="Textarea">
    <InputGroup label="Note">
      <InputGroupTextarea model-value="Ship the release." />
    </InputGroup>
  </variant>
  <variant name="Disabled">
    <InputGroup label="Website" disabled>
      <InputGroupAddon>https://</InputGroupAddon>
      <InputGroupInput model-value="vize.dev" />
      <InputGroupButton>Copy</InputGroupButton>
    </InputGroup>
  </variant>
</art>
