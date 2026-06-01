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
  await expect(
    page.getByRole("heading", { name: /Prototype work item review/i }),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: /Approve plan/i }),
  ).toBeVisible();
  await expect(
    page.getByText(/BOG-571 has no final implementation evidence/i),
  ).toBeVisible();
  await expect(
    page.getByText(/\[REDACTED\] placeholders stand in/i),
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

test("review control plan and final decisions are selectable", async ({
  page,
}) => {
  await page.goto("/");

  await page.getByRole("button", { name: /Approve plan/i }).click();
  await expect(page.getByText("Plan approved")).toBeVisible();

  await page.getByRole("button", { name: /Send to rework/i }).click();
  await expect(page.getByText("Rework selected")).toBeVisible();
});

test("review packet shows redacted placeholders and hides secret-shaped values", async ({
  page,
}) => {
  await page.goto("/");

  await expect(
    page.getByText(/\[REDACTED\] placeholders stand in/i),
  ).toBeVisible();

  const secretExposure = await page.evaluate(() => {
    const text = document.body.textContent ?? "";
    return /(sk-[A-Za-z0-9]{20,}|ghp_[A-Za-z0-9]{20,}|AKIA[0-9A-Z]{16}|-----BEGIN [A-Z ]+PRIVATE KEY-----)/.test(
      text,
    );
  });

  expect(secretExposure).toBe(false);
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

test("review packet visual smoke has no horizontal overflow", async ({
  page,
}) => {
  await page.goto("/");

  const layout = await page.locator(".review-control").evaluate((surface) => {
    const rect = surface.getBoundingClientRect();
    const overflowing = Array.from(surface.querySelectorAll("*")).filter(
      (element) => {
        const childRect = element.getBoundingClientRect();
        return (
          childRect.left < rect.left - 1 || childRect.right > rect.right + 1
        );
      },
    );

    return {
      width: rect.width,
      overflowing: overflowing.length,
      horizontalScroll: surface.scrollWidth > surface.clientWidth + 1,
    };
  });

  expect(layout.width).toBeGreaterThan(0);
  expect(layout.overflowing).toBe(0);
  expect(layout.horizontalScroll).toBe(false);

  await page.screenshot({
    path: "test-results/review-control-prototype.png",
    fullPage: true,
  });
});
