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

test("first-run onboarding exposes safe policy-check diagnostic contract", async ({
  page,
}) => {
  const requests: string[] = [];
  page.on("request", (request) => requests.push(request.url()));

  await page.goto("/");

  await expect(
    page.getByRole("heading", { name: /First-run admin onboarding/i }),
  ).toBeVisible();
  await expect(
    page.getByRole("list", { name: /First-run readiness steps/i }),
  ).toContainText("Safe diagnostic run");
  await expect(page.getByRole("textbox", { name: "Provider" })).toHaveValue(
    "Gateway policy route",
  );
  await expect(
    page.getByText("fixture-taskotter-policy-check-model"),
  ).toBeVisible();
  await expect(
    page.getByText("credref_01J9Z4P4BS0M9P2QJ6T8Z6W2EP"),
  ).toBeVisible();
  await expect(page.getByText("Allow paid call")).toBeVisible();
  await expect(page.getByText("Idempotency-Key")).toBeVisible();
  await expect(
    page.getByText("usage_01J9Z4P4BS0M9P2QJ6T8Z6W2EP", { exact: true }),
  ).toBeVisible();
  const contract = page.getByRole("region", { name: "Diagnostic contract" });
  await expect(contract.getByText("Usage delta")).toBeVisible();
  await expect(contract.getByText("Billable")).toBeVisible();
  await expect(page.getByText("Paid activation required")).toBeVisible();

  await page.getByRole("button", { name: /Run safe diagnostic/i }).click();
  await expect(page.getByRole("status")).toContainText("Diagnostic complete");
  await expect(
    page.getByRole("heading", { name: /Diagnostic result timeline/i }),
  ).toBeVisible();
  await expect(page.getByText(/Runner or MCP availability/i)).toBeVisible();
  await expect(page.getByText(/redacted=true/i)).toHaveCount(4);

  expect(requests.some((url) => /provider|gateway|runner/i.test(url))).toBe(
    false,
  );
});

test("first-run onboarding redacts secret-shaped fixture candidates", async ({
  page,
}) => {
  await page.goto("/");

  const forbidden = [
    "sk-taskotter-fixture-secret-token-value",
    "TASKOTTER_PROVIDER_API_KEY",
    "sk-live-unsafe-value",
    "Summarize customer private payload",
  ];

  for (const value of forbidden) {
    await expect(page.locator("body")).not.toContainText(value);
  }

  const exposedAttributes = await page.evaluate(() =>
    Array.from(document.querySelectorAll("*")).flatMap((element) =>
      ["aria-label", "aria-description", "title"].map(
        (attribute) => element.getAttribute(attribute) ?? "",
      ),
    ),
  );
  const storageDump = await page.evaluate(() =>
    JSON.stringify({
      local: Object.fromEntries(
        Array.from({ length: localStorage.length }, (_, index) => {
          const key = localStorage.key(index) ?? "";
          return [key, localStorage.getItem(key)];
        }),
      ),
      session: Object.fromEntries(
        Array.from({ length: sessionStorage.length }, (_, index) => {
          const key = sessionStorage.key(index) ?? "";
          return [key, sessionStorage.getItem(key)];
        }),
      ),
    }),
  );

  for (const value of forbidden) {
    expect(exposedAttributes.join(" ")).not.toContain(value);
    expect(storageDump).not.toContain(value);
  }
});

test("first-run onboarding handles validation, read-only preview, keyboard, and mobile bounds", async ({
  isMobile,
  page,
  viewport,
}) => {
  await page.goto("/");

  await expect(
    page.getByRole("button", { name: "Admin fixture" }),
  ).toHaveAttribute("aria-pressed", "true");
  await expect(
    page.getByRole("button", { name: /Run safe diagnostic/i }),
  ).toBeEnabled();

  await page.getByRole("button", { name: "Member fixture" }).click();
  await expect(page.getByText(/Members can inspect readiness/i)).toBeVisible();
  await expect(
    page.getByRole("button", { name: /Save readiness/i }),
  ).toBeDisabled();
  await expect(
    page.getByRole("button", { name: /Run safe diagnostic/i }),
  ).toBeDisabled();

  await page.getByRole("button", { name: "Admin fixture" }).click();
  const provider = page.getByRole("textbox", { name: "Provider" });
  await provider.fill("");
  await page.getByRole("button", { name: /Save readiness/i }).click();
  await expect(page.getByRole("alert")).toContainText(
    "Provider route is required",
  );
  await expect(page.getByRole("alert")).toBeFocused();
  await expect(provider).toHaveAttribute("aria-invalid", "true");

  if (!isMobile && (viewport?.width ?? 0) >= 768) {
    await page
      .getByRole("link", { name: "Provider route is required." })
      .click();
    await expect(provider).toBeFocused();
    await page.keyboard.press("Tab");
    await expect(
      page.getByRole("button", { name: "Admin fixture" }),
    ).toBeFocused();
    await page.keyboard.press("Tab");
    await expect(
      page.getByRole("button", { name: "Member fixture" }),
    ).toBeFocused();
    await page.keyboard.press("Tab");
    await expect(
      page.getByRole("button", { name: /Save readiness/i }),
    ).toBeFocused();
    await page.keyboard.press("Tab");
    await expect(
      page.getByRole("button", { name: /Run safe diagnostic/i }),
    ).toBeFocused();
  }

  await page.setViewportSize({ width: 390, height: 900 });
  const panelBounds = await page
    .locator(".first-run-panel")
    .evaluate((panel) => {
      const panelRect = panel.getBoundingClientRect();
      const children = Array.from(
        panel.querySelectorAll("button, input, dd, li"),
      );
      const tolerance = 1;
      return children.every((child) => {
        const rect = child.getBoundingClientRect();
        return (
          rect.width > 0 &&
          rect.left >= panelRect.left - tolerance &&
          rect.right <= panelRect.right + tolerance
        );
      });
    });

  expect(panelBounds).toBe(true);
});
