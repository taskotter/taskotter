import { createI18n, resolveLocalePreferences } from ".";

describe("i18n locale resolution", () => {
  it("uses user preference before Working Group default and browser suggestion", () => {
    expect(
      resolveLocalePreferences({
        userLanguage: "en-XA",
        workingGroupDefaultLanguage: "en",
        browserLanguage: "fr-FR",
      }),
    ).toMatchObject({ contentLocale: "en-XA", source: "user" });
  });

  it("uses Working Group default before browser suggestion", () => {
    expect(
      resolveLocalePreferences({
        workingGroupDefaultLanguage: "en-XA",
        browserLanguage: "en-US",
      }),
    ).toMatchObject({ contentLocale: "en-XA", source: "working-group" });
  });

  it("falls back deterministically when no supported locale is available", () => {
    expect(
      resolveLocalePreferences({
        userLanguage: "fr-FR",
        workingGroupDefaultLanguage: "de-DE",
        browserLanguage: "ko-KR",
      }),
    ).toMatchObject({ contentLocale: "en", fallbackLocale: "en" });
  });
});

describe("i18n resource smoke fixtures", () => {
  it("renders pseudo-localized expanded UI copy", () => {
    const i18n = createI18n(
      resolveLocalePreferences({ userLanguage: "en-XA" }),
    );

    expect(i18n.t("webShell.page.title")).toContain("[!!");
    expect(i18n.t("webShell.page.title").length).toBeGreaterThan(
      "Issue operations".length,
    );
  });

  it("hides missing keys behind a localized fallback string", () => {
    const i18n = createI18n(resolveLocalePreferences({ userLanguage: "en" }));

    expect(i18n.t("issues.unknown.key")).toBe("Content unavailable.");
  });

  it("supports pluralized and interpolated smoke strings", () => {
    const i18n = createI18n(resolveLocalePreferences({ userLanguage: "en" }));

    expect(i18n.plural("issues.row.comments", 1)).toBe("1 comment");
    expect(i18n.plural("issues.row.comments", 3)).toBe("3 comments");
  });

  it("formats dates with explicit formatting locale and timezone", () => {
    const i18n = createI18n(
      resolveLocalePreferences({
        userLanguage: "en",
        formattingLocale: "en-US",
        timeZone: "UTC",
      }),
    );

    expect(i18n.formatDateTime("2026-06-01T12:30:00Z")).toMatch(
      /Jun 01, 12:30 PM/,
    );
  });

  it("formats numbers with the locale preference separate from content locale", () => {
    const i18n = createI18n(
      resolveLocalePreferences({
        userLanguage: "en-XA",
        formattingLocale: "de-DE",
      }),
    );

    expect(i18n.formatNumber(1200)).toBe("1.200");
  });
});
