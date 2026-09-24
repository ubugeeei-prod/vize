import { normalizeRotation, rotatedBounds } from "./image-cropper-geometry.ts";
import type {
  CropArea,
  CropImageOptions,
  CropImageOutput,
  ImageCropperErrorCode,
} from "./image-cropper-types.ts";

/** Typed error thrown by {@link cropImage}. `code` is stable across releases. */
export class ImageCropperError extends Error {
  /** Stable diagnostic code. */
  readonly code: ImageCropperErrorCode;

  constructor(code: ImageCropperErrorCode, message: string) {
    super(`${code}: ${message}`);
    this.name = "ImageCropperError";
    this.code = code;
  }
}

function loadImage(
  source: HTMLImageElement | string,
  crossOrigin: CropImageOptions["crossOrigin"],
): Promise<HTMLImageElement> {
  const image = typeof source === "string" ? document.createElement("img") : source;
  if (image.complete && image.naturalWidth > 0) return Promise.resolve(image);
  return new Promise((resolve, reject) => {
    const cleanup = (): void => {
      image.removeEventListener("load", onLoad);
      image.removeEventListener("error", onError);
    };
    const onLoad = (): void => {
      cleanup();
      resolve(image);
    };
    const onError = (): void => {
      cleanup();
      reject(new ImageCropperError("VIZE_UI_IMAGE_CROPPER_IMAGE_LOAD", "the image failed to load"));
    };
    image.addEventListener("load", onLoad);
    image.addEventListener("error", onError);
    if (typeof source === "string") {
      image.crossOrigin = crossOrigin ?? "anonymous";
      image.src = source;
    }
  });
}

function encode(
  canvas: HTMLCanvasElement,
  output: CropImageOutput,
  options: CropImageOptions,
): Promise<Blob | string> {
  const type = options.type ?? "image/png";
  if (output === "data-url") return Promise.resolve(canvas.toDataURL(type, options.quality));
  return new Promise((resolve, reject) => {
    canvas.toBlob(
      (blob) => {
        if (blob === null) {
          reject(
            new ImageCropperError(
              "VIZE_UI_IMAGE_CROPPER_ENCODE_FAILED",
              `could not encode ${type}`,
            ),
          );
        } else {
          resolve(blob);
        }
      },
      type,
      options.quality,
    );
  });
}

async function render(
  source: HTMLImageElement | string,
  crop: CropArea,
  options: CropImageOptions,
  output: CropImageOutput,
): Promise<Blob | string> {
  if (typeof document === "undefined") {
    throw new ImageCropperError(
      "VIZE_UI_IMAGE_CROPPER_CANVAS_UNAVAILABLE",
      "cropping requires a browser canvas",
    );
  }
  const x = Math.round(crop.x);
  const y = Math.round(crop.y);
  const width = Math.round(crop.width);
  const height = Math.round(crop.height);
  if (!(width >= 1 && height >= 1)) {
    throw new ImageCropperError("VIZE_UI_IMAGE_CROPPER_EMPTY_CROP", "the crop area is empty");
  }
  const image = await loadImage(source, options.crossOrigin);
  const scale = Math.min(
    1,
    options.maxWidth !== undefined && options.maxWidth > 0 ? options.maxWidth / width : 1,
    options.maxHeight !== undefined && options.maxHeight > 0 ? options.maxHeight / height : 1,
  );
  const canvas = document.createElement("canvas");
  canvas.width = Math.max(1, Math.round(width * scale));
  canvas.height = Math.max(1, Math.round(height * scale));
  const context = canvas.getContext("2d");
  if (context === null) {
    throw new ImageCropperError(
      "VIZE_UI_IMAGE_CROPPER_CANVAS_UNAVAILABLE",
      "the canvas 2D context is unavailable",
    );
  }
  const rotation = normalizeRotation(options.rotation ?? 0);
  const natural = { width: image.naturalWidth, height: image.naturalHeight };
  const bounds = rotatedBounds(natural, rotation);
  context.scale(scale, scale);
  context.translate(-x + bounds.width / 2, -y + bounds.height / 2);
  context.rotate((rotation * Math.PI) / 180);
  context.drawImage(image, -natural.width / 2, -natural.height / 2);
  return await encode(canvas, output, options);
}

/**
 * Crop an image with a canvas: the image is rotated into its bounding box, the
 * crop area is cut out, and the result is optionally downscaled and encoded.
 *
 * Client-only. Rejects with a typed {@link ImageCropperError} when no canvas is
 * available, the crop is empty, the image fails to load, or encoding fails.
 */
export function cropImage(
  source: HTMLImageElement | string,
  crop: CropArea,
  options: CropImageOptions & { readonly output: "data-url" },
): Promise<string>;
export function cropImage(
  source: HTMLImageElement | string,
  crop: CropArea,
  options?: CropImageOptions & { readonly output?: "blob" },
): Promise<Blob>;
export async function cropImage(
  source: HTMLImageElement | string,
  crop: CropArea,
  options: CropImageOptions & { readonly output?: CropImageOutput } = {},
): Promise<Blob | string> {
  return await render(source, crop, options, options.output ?? "blob");
}
