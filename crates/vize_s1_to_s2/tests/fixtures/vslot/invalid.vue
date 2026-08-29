<span v-slot="misplaced">x</span>
<Modal v-slot="own">
  <template #late>y</template>
</Modal>
<Grid>
  <template #cell="c">a</template>
  <template #cell>b</template>
  <template #default>c</template>
  <hr>
</Grid>
