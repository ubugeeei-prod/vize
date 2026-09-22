<template>
  <div v-bind:id="id" v-on:click="onClick" v-bind:[name]="value" v-bind="row"></div>
  <button :disabled="true" v-bind:type="kind"></button>
</template>
