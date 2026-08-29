<ul>
  <li v-for="(item, i) in items">
    <input v-model="item">
    <input v-model:value="item.name">
  </li>
</ul>
<Panel v-slot="row">
  <textarea v-model="row"></textarea>
</Panel>
<slot v-slot="p"></slot>
