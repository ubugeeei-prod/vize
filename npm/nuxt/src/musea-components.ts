export const MUSEA_ART_COMPONENT_IGNORE = "**/*.art.vue";

export type MuseaNuxtComponentDir =
  | string
  | {
      ignore?: string[];
      path: string;
      [key: string]: unknown;
    };

export function appendMuseaArtComponentIgnore(dirs: MuseaNuxtComponentDir[]): void {
  for (const [index, dir] of dirs.entries()) {
    if (typeof dir === "string") {
      dirs[index] = {
        path: dir,
        ignore: [MUSEA_ART_COMPONENT_IGNORE],
      };
      continue;
    }

    const ignore = Array.isArray(dir.ignore) ? dir.ignore : [];
    if (ignore.includes(MUSEA_ART_COMPONENT_IGNORE)) {
      continue;
    }

    dir.ignore = [...ignore, MUSEA_ART_COMPONENT_IGNORE];
  }
}
