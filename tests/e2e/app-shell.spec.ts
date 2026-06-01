import { expect, test } from "@playwright/test";

test("app shell, issue detail, run progress, and setup first path render", async ({
  page,
}) => {
  await page.goto("/");

  await expect(
    page.getByRole("navigation", { name: /taskotter navigation/i }),
  ).toBeVisible();
  await expect(
    page.getByRole("main", { name: /issue operations/i }),
  ).toBeVisible();
  await expect(
    page.getByRole("complementary", { name: /BOG-436/i }),
  ).toBeVisible();
  await expect(
    page.getByRole("heading", { name: /Working Group setup path/i }),
  ).toBeVisible();
  await expect(
    page.getByRole("heading", { name: /^Assigned\b/i }),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: /BOG-436 Build MVP app shell/i }),
  ).toBeVisible();
  await expect(
    page.getByRole("heading", { name: /Agent run progress/i }),
  ).toBeVisible();
  await expect(
    page.getByRole("heading", { name: /Threaded comments/i }),
  ).toBeVisible();
  await expect(page.getByRole("textbox", { name: "Reply" })).toBeVisible();
});

test("keyboard focus reaches issue rows and composer", async ({ page }) => {
  await page.goto("/");

  const selectedIssue = page.getByRole("button", {
    name: /BOG-436 Build MVP app shell/i,
  });
  await selectedIssue.focus();
  await expect(selectedIssue).toBeFocused();

  await page.getByRole("textbox", { name: "Reply" }).focus();
  await expect(page.getByRole("textbox", { name: "Reply" })).toBeFocused();
});

test("responsive sidebar controls keep accessible names", async ({ page }) => {
  await page.goto("/");

  await expect(
    page.getByRole("button", {
      name: /Switch working group: Platform Delivery/i,
    }),
  ).toBeVisible();

  for (const name of ["Inbox", "Issues", "Chat", "Agents"]) {
    await expect(page.getByRole("link", { name })).toBeVisible();
  }
});

test("issue row metadata chips stay visible inside the workspace", async ({
  page,
}) => {
  await page.goto("/");

  const selectedIssue = page.locator('article[aria-label^="BOG-436"]');
  await expect(selectedIssue.getByText("In progress")).toBeVisible();
  await expect(selectedIssue.getByText("Policy OK")).toBeVisible();
  await expect(selectedIssue.getByText("4 comments")).toBeVisible();

  const layout = await selectedIssue.evaluate((issueRow) => {
    const surface = issueRow.closest(".issue-surface");
    const meta = issueRow.querySelector(".issue-row-meta");
    const chips = Array.from(issueRow.querySelectorAll(".issue-row-meta > *"));

    if (!surface || !meta || chips.length === 0) {
      return { missing: true };
    }

    const surfaceRect = surface.getBoundingClientRect();
    const metaRect = meta.getBoundingClientRect();
    const chipRects = chips.map((chip) => chip.getBoundingClientRect());
    const tolerance = 1;

    return {
      missing: false,
      metaInsideSurface:
        metaRect.left >= surfaceRect.left - tolerance &&
        metaRect.right <= surfaceRect.right + tolerance,
      chipsInsideSurface: chipRects.every(
        (rect) =>
          rect.width > 0 &&
          rect.left >= surfaceRect.left - tolerance &&
          rect.right <= surfaceRect.right + tolerance,
      ),
    };
  });

  expect(layout).toEqual({
    missing: false,
    metaInsideSurface: true,
    chipsInsideSurface: true,
  });
});

test("pseudo locale renders system copy and de-DE formatting while preserving user-authored issue content", async ({
  page,
}) => {
  await page.addInitScript(() => {
    window.localStorage.setItem(
      "taskotter.localePreferences",
      JSON.stringify({ userLanguage: "en-XA", formattingLocale: "de-DE" }),
    );
  });

  await page.goto("/");

  await expect(
    page.getByRole("main", { name: /\[!! Ïssüë õpëràtïõns !!\]/ }),
  ).toBeVisible();
  await expect(
    page.getByRole("heading", {
      name: /\[!! Wõrkïng Grõüp sëtüp pàth !!\]/,
    }),
  ).toBeVisible();
  await expect(
    page.getByRole("heading", { name: /\[!! Àgënt rün prõgrëss !!\]/ }),
  ).toBeVisible();
  await expect(page.getByText(/30\. Mai, 09:04/).first()).toBeVisible();
  await expect(page.getByText(/\[!! 1\.200ms, 2 tõõls !!\]/)).toBeVisible();

  await expect(
    page.getByRole("heading", {
      name: "Build MVP app shell, issue workspace, and focus panel UI",
    }),
  ).toBeVisible();
  await expect(
    page.getByText(
      "Use the UX/IA design comment and QA gate as the implementation source. Keep API contracts replaceable until BOG-425 lands.",
    ),
  ).toBeVisible();
});
