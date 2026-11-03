<form :action="act" @submit.prevent="save()">
  <input type="text" v-model.trim="draft.note" :key="draft.id" .value="raw">
  <Field v-model:title="doc.title" :model-value></Field>
  <slot name="row" tone="brisk" :item="row" @pick="choose(row)"></slot>
  <button v-bind="extra" v-on="handlers" @[evt]="run">go</button>
</form>
