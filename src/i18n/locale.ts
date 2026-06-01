export const fallbackLocale = "en" as const;

export const supportedContentLocales = ["en", "en-XA"] as const;

export type SupportedContentLocale = (typeof supportedContentLocales)[number];

export type LocaleSource = "user" | "working-group" | "browser" | "fallback";

export interface LocalePreferences {
  userLanguage?: string;
  workingGroupDefaultLanguage?: string;
  browserLanguage?: string;
  formattingLocale?: string;
  timeZone?: string;
}

export interface ResolvedLocale {
  contentLocale: SupportedContentLocale;
  fallbackLocale: typeof fallbackLocale;
  formattingLocale: string;
  timeZone: string;
  source: LocaleSource;
}

function normalizeContentLocale(
  value: string | undefined,
): SupportedContentLocale | undefined {
  if (!value) return undefined;
  const normalized = value.trim();
  if ((supportedContentLocales as readonly string[]).includes(normalized)) {
    return normalized as SupportedContentLocale;
  }

  const language = normalized.split("-")[0]?.toLowerCase();
  return supportedContentLocales.find((locale) => locale === language);
}

function browserLanguage(): string | undefined {
  if (typeof navigator === "undefined") return undefined;
  return navigator.languages?.[0] ?? navigator.language;
}

function defaultTimeZone(): string {
  try {
    return Intl.DateTimeFormat().resolvedOptions().timeZone || "UTC";
  } catch {
    return "UTC";
  }
}

export function resolveLocalePreferences(
  preferences: LocalePreferences = {},
): ResolvedLocale {
  const candidates: Array<[LocaleSource, string | undefined]> = [
    ["user", preferences.userLanguage],
    ["working-group", preferences.workingGroupDefaultLanguage],
    ["browser", preferences.browserLanguage ?? browserLanguage()],
    ["fallback", fallbackLocale],
  ];

  const [source, contentLocale] = candidates
    .map(([candidateSource, value]) => [
      candidateSource,
      normalizeContentLocale(value),
    ])
    .find(([, value]) => Boolean(value)) as [
    LocaleSource,
    SupportedContentLocale,
  ];

  return {
    contentLocale,
    fallbackLocale,
    formattingLocale: preferences.formattingLocale ?? contentLocale,
    timeZone: preferences.timeZone ?? defaultTimeZone(),
    source,
  };
}
