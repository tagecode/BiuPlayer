import type { MessageKey } from "./i18n";

const codeKeys: Record<string, MessageKey> = {
  E_INVALID_ARGUMENT: "error.E_INVALID_ARGUMENT",
  E_AUDIO_DEVICE: "error.E_AUDIO_DEVICE",
  E_INTERNAL: "error.E_INTERNAL",
  E_WINAMP: "error.E_WINAMP",
};

type Translate = (key: MessageKey, vars?: Record<string, string | number>) => string;

export function formatError(error: unknown, t: Translate): string {
  if (typeof error === "string") return error;
  if (error && typeof error === "object") {
    const value = error as { code?: unknown; message?: unknown };
    const code = typeof value.code === "string" ? value.code : "";
    if (code && codeKeys[code]) return t(codeKeys[code]);
    if (typeof value.message === "string" && value.message) {
      return code ? `${code}: ${value.message}` : value.message;
    }
  }
  return t("error.unknown");
}
