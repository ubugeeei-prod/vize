<Card>
  <template #head="{ icon, title }"><b>{{ title }}</b></template>
  <template v-slot:body.raw="row"><p>{{ row.text }}</p></template>
  <p>trailing body</p>
</Card>
<Panel v-slot="props"><em>{{ props.x }}</em></Panel>
<List>
  <template #[cell]="value"><i>{{ value }}</i></template>
</List>
