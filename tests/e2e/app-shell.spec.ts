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
