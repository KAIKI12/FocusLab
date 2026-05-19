import { convertFileSrc } from "@tauri-apps/api/core";

const imageSrcCache = new Map<string, string>();

export function getInspirationImageSrc(imagePath: string | null | undefined): string {
  if (!imagePath) return "";
  const cached = imageSrcCache.get(imagePath);
  if (cached) return cached;

  const src = convertFileSrc(imagePath);
  imageSrcCache.set(imagePath, src);
  return src;
}
