export type TranslationResource = Record<string, string>;

export type TranslationNamespace =
  | "chat"
  | "commonErrors"
  | "issues"
  | "settings"
  | "webShell";

export type TranslationResources = Record<
  TranslationNamespace,
  TranslationResource
>;

export type TranslationKey = `${TranslationNamespace}.${string}`;

export type TranslationValues = Record<string, string | number>;
