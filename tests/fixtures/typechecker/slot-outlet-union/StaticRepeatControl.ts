import type { Slots } from './StaticRepeat.vue';
type Payload = Parameters<NonNullable<Slots['panel']>>[0];
type IfEquals<T, U> = (<G>() => G extends T ? 1 : 2) extends
  (<G>() => G extends U ? 1 : 2) ? true : false;
const exactKind: IfEquals<Payload['kind'], string> = true;
const arbitrary: Payload = { kind: 'arbitrary' };
void [exactKind, arbitrary];
