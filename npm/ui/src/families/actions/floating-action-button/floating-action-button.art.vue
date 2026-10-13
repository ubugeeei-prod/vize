<script setup lang="ts">
import {
  FloatingActionButton,
  SpeedDial,
  SpeedDialAction,
  SpeedDialContent,
  SpeedDialTrigger,
} from "./floating-action-button.ts";
</script>

<art title="Floating Action Button" status="ready">
  <variant name="Icon" default>
    <FloatingActionButton placement="bottom-end" aria-label="Compose">+</FloatingActionButton>
  </variant>
  <variant name="Extended">
    <FloatingActionButton placement="bottom-end" extended>Compose</FloatingActionButton>
  </variant>
  <variant name="Disabled">
    <FloatingActionButton placement="bottom-start" extended disabled>Compose</FloatingActionButton>
  </variant>
  <variant name="Speed dial">
    <SpeedDial direction="up">
      <SpeedDialTrigger aria-label="Create">+</SpeedDialTrigger>
      <SpeedDialContent>
        <SpeedDialAction value="note" label="New note">N</SpeedDialAction>
        <SpeedDialAction value="task" label="New task">T</SpeedDialAction>
      </SpeedDialContent>
    </SpeedDial>
  </variant>
  <variant name="Speed dial open">
    <SpeedDial direction="up" default-open>
      <SpeedDialTrigger aria-label="Create">+</SpeedDialTrigger>
      <SpeedDialContent>
        <SpeedDialAction value="note" label="New note">N</SpeedDialAction>
        <SpeedDialAction value="task" label="New task">T</SpeedDialAction>
      </SpeedDialContent>
    </SpeedDial>
  </variant>
</art>
