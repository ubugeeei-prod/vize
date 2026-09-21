<script setup lang="ts">
import { GetStudyHistoriesQuery } from './schema'
import { UnwrapArray } from './types'

type StudyHistory = UnwrapArray<
  GetStudyHistoriesQuery['student']['studyHistories']
>

defineProps<{ studyHistory: StudyHistory }>()
</script>

<template>
  <article>{{ studyHistory }}</article>
</template>