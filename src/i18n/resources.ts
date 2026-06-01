import { en } from "./resources/en";
import type { TranslationResources } from "./types";
import type { SupportedContentLocale } from "./locale";

const accentMap: Record<string, string> = {
  a: "à",
  e: "ë",
  i: "ï",
  o: "õ",
  u: "ü",
  A: "À",
  E: "Ë",
  I: "Ï",
  O: "Õ",
  U: "Ü",
};

function pseudoLocalize(value: string): string {
  return `[!! ${value.replace(/[aeiouAEIOU]/g, (char) => accentMap[char] ?? char)} !!]`;
}

function pseudoNamespace(
  namespace: TranslationResources[keyof TranslationResources],
): TranslationResources[keyof TranslationResources] {
  return Object.fromEntries(
    Object.entries(namespace).map(([key, value]) => [
      key,
      pseudoLocalize(value),
    ]),
  );
}

const pseudoEnglish: TranslationResources = {
  chat: pseudoNamespace(en.chat),
  commonErrors: pseudoNamespace(en.commonErrors),
  issues: pseudoNamespace(en.issues),
  settings: pseudoNamespace(en.settings),
  webShell: pseudoNamespace(en.webShell),
};

export const resources: Record<SupportedContentLocale, TranslationResources> = {
  en,
  "en-XA": pseudoEnglish,
};
