import {
  fallbackLocale,
  type ResolvedLocale,
  type SupportedContentLocale,
} from "./locale";
import { resources } from "./resources";
import type {
  TranslationKey,
  TranslationNamespace,
  TranslationValues,
} from "./types";

export type { LocalePreferences, ResolvedLocale } from "./locale";
export { resolveLocalePreferences } from "./locale";

function splitKey(key: TranslationKey): [TranslationNamespace, string] {
  const [namespace, ...rest] = key.split(".");
  return [namespace as TranslationNamespace, rest.join(".")];
}

function interpolate(value: string, values: TranslationValues = {}): string {
  return value.replace(/\{(\w+)\}/g, (_, name: string) =>
    String(values[name] ?? `{${name}}`),
  );
}

function lookup(locale: SupportedContentLocale, key: TranslationKey) {
  const [namespace, resourceKey] = splitKey(key);
  return resources[locale][namespace]?.[resourceKey];
}

export function createI18n(resolvedLocale: ResolvedLocale) {
  const t = (key: TranslationKey, values?: TranslationValues) => {
    const value =
      lookup(resolvedLocale.contentLocale, key) ??
      lookup(fallbackLocale, key) ??
      lookup(fallbackLocale, "commonErrors.missingTranslation");

    return interpolate(value ?? "Content unavailable.", values);
  };

  return {
    locale: resolvedLocale,
    t,
    plural(
      key: TranslationKey,
      count: number,
      values: TranslationValues = {},
    ): string {
      const suffix = count === 1 ? "one" : "other";
      return t(`${key}.${suffix}` as TranslationKey, { ...values, count });
    },
    formatDateTime(value: string | Date): string {
      const date = value instanceof Date ? value : new Date(value);
      if (Number.isNaN(date.valueOf())) return String(value);

      return new Intl.DateTimeFormat(resolvedLocale.formattingLocale, {
        month: "short",
        day: "2-digit",
        hour: "2-digit",
        minute: "2-digit",
        timeZone: resolvedLocale.timeZone,
      }).format(date);
    },
  };
}
