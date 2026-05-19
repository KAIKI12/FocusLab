import { onBeforeUnmount, ref } from "vue";

const MAX_IMAGE_BYTES = 8 * 1024 * 1024;
const MAX_IMAGE_SIZE_MB = MAX_IMAGE_BYTES / 1024 / 1024;
const SUPPORTED_IMAGE_MIME_TYPES = new Set([
  "image/png",
  "image/jpeg",
  "image/webp",
  "image/gif",
]);

export interface InspirationImageUpload {
  bytes: number[];
  mimeType: string;
}

export interface InspirationImageDraftState {
  blob: Blob;
  mimeType: string;
  previewUrl: string;
}

function extractClipboardImage(event: ClipboardEvent): Blob | null {
  const items = event.clipboardData?.items;
  if (!items?.length) return null;

  for (const item of items) {
    if (!item.type.startsWith("image/")) continue;
    const file = item.getAsFile();
    if (file) return file;
  }
  return null;
}

export function useInspirationImageDraft() {
  const imageDraft = ref<InspirationImageDraftState | null>(null);
  const imageError = ref("");

  function validateImage(blob: Blob): string {
    const mimeType = blob.type || "image/png";
    if (!SUPPORTED_IMAGE_MIME_TYPES.has(mimeType)) {
      return "暂不支持这种图片格式";
    }
    if (blob.size > MAX_IMAGE_BYTES) {
      return `图片不能超过 ${MAX_IMAGE_SIZE_MB} MB`;
    }
    return "";
  }

  function clearImage() {
    if (imageDraft.value?.previewUrl) {
      URL.revokeObjectURL(imageDraft.value.previewUrl);
    }
    imageDraft.value = null;
    imageError.value = "";
  }

  function rejectImage(message: string) {
    clearImage();
    imageError.value = message;
  }

  function setImage(blob: Blob): boolean {
    const validationError = validateImage(blob);
    if (validationError) {
      rejectImage(validationError);
      return false;
    }
    clearImage();
    imageDraft.value = {
      blob,
      mimeType: blob.type || "image/png",
      previewUrl: URL.createObjectURL(blob),
    };
    return true;
  }

  function handlePasteImage(event: ClipboardEvent): boolean {
    const blob = extractClipboardImage(event);
    if (!blob) return false;
    event.preventDefault();
    return setImage(blob);
  }

  async function toUploadPayload(): Promise<InspirationImageUpload | null> {
    if (!imageDraft.value) return null;
    const validationError = validateImage(imageDraft.value.blob);
    if (validationError) {
      rejectImage(validationError);
      return null;
    }
    const buffer = await imageDraft.value.blob.arrayBuffer();
    return {
      bytes: Array.from(new Uint8Array(buffer)),
      mimeType: imageDraft.value.mimeType,
    };
  }

  onBeforeUnmount(() => {
    clearImage();
  });

  return {
    imageDraft,
    imageError,
    setImage,
    clearImage,
    handlePasteImage,
    toUploadPayload,
  };
}
