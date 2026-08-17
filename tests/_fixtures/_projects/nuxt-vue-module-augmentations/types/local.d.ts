// Hand-written project augmentation. Same shape as the generated one, in a
// directory the app owns.
declare module "vue" {
  interface ComponentCustomProperties {
    $local: (label: string) => string;
  }
}

export {};
