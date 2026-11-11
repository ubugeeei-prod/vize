export interface CardSlots {
  default(): unknown;
  footer(props: { count: number }): unknown;
}

export interface CardEmits {
  close: [];
}
