import type { Slots } from './Optional.vue';
type Payload = Parameters<NonNullable<Slots['panel']>>[0];
export const absent: Payload = {};
export const text: Payload = { item: 'x' };
export const number: Payload = { item: 1 };
export const wrong: Payload = { item: true };
