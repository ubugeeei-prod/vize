<template>
  <div :id="id" @click="onClick" :[name]="value" v-bind="row"></div>
  <button :disabled="true" :type="kind"></button>
</template>
