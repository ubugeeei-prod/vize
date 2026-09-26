<article v-custom="cfg">
  <header><em>fixed</em></header>
  <template v-if="ready"><p>on</p></template>
  <li v-for="item in items"><u>row</u></li>
  <Card><template #cell><s>c</s></template><slot name="tail"></slot></Card>
</article>
