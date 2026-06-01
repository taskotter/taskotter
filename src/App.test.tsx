import { fireEvent, render, screen, within } from "@testing-library/react";
import { App } from "./App";
import { taskotterConsoleFixture } from "./data/taskotterFixtures";

describe("TaskOtter console", () => {
  it("renders the app shell, grouped issues, focus panel, comments, and setup path", async () => {
    render(<App />);

    expect(
      await screen.findByRole("navigation", { name: /taskotter navigation/i }),
    ).toBeVisible();
    expect(
      screen.getByRole("main", { name: /issue operations/i }),
    ).toBeVisible();
    expect(
      screen.getByRole("complementary", { name: /BOG-436/i }),
    ).toBeVisible();

    expect(
      screen.getByRole("heading", { name: /Working Group setup path/i }),
    ).toBeVisible();
    expect(screen.getByRole("heading", { name: /^Assigned\b/i })).toBeVisible();
    expect(
      screen.getByText(/typed local data adapter remains replaceable/i),
    ).toBeVisible();
    expect(
      screen.getByRole("heading", { name: /Agent run progress/i }),
    ).toBeVisible();
    expect(
      screen.getByRole("heading", { name: /Threaded comments/i }),
    ).toBeVisible();
    expect(
      screen.getByRole("heading", { name: /Prototype work item review/i }),
    ).toBeVisible();
    expect(screen.getByRole("button", { name: /Approve plan/i })).toBeVisible();
    expect(screen.getByRole("button", { name: /Mark done/i })).toBeVisible();
    expect(
      screen.getByText(/BOG-571 has no final implementation evidence/i),
    ).toBeVisible();
    expect(
      screen.getByText(/\[REDACTED\] placeholders stand in/i),
    ).toBeVisible();
    expect(document.body.textContent ?? "").not.toMatch(
      /(sk-[A-Za-z0-9]{20,}|ghp_[A-Za-z0-9]{20,}|AKIA[0-9A-Z]{16}|-----BEGIN [A-Z ]+PRIVATE KEY-----)/,
    );

    const composer = screen.getByRole("form", { name: /reply composer/i });
    expect(within(composer).getByLabelText("Reply")).toBeVisible();
  });

  it("updates plan approval and done/rework decisions inside the review control prototype", async () => {
    render(<App />);

    await screen.findByRole("heading", {
      name: /Prototype work item review/i,
    });

    fireEvent.click(screen.getByRole("button", { name: /Approve plan/i }));
    expect(await screen.findByText("Plan approved")).toBeVisible();

    fireEvent.click(screen.getByRole("button", { name: /Send to rework/i }));
    expect(await screen.findByText("Rework selected")).toBeVisible();
  });

  it("renders pseudo-localized accessible shell labels without raw missing keys", async () => {
    const originalPreferences = taskotterConsoleFixture.localePreferences;
    taskotterConsoleFixture.localePreferences = {
      ...originalPreferences,
      userLanguage: "en-XA",
      formattingLocale: "en-US",
    };

    try {
      render(<App />);

      expect(
        await screen.findByText(/\[!! Ïssüë õpëràtïõns !!\]/),
      ).toBeVisible();
      expect(
        screen.getByRole("navigation", {
          name: /\[!! TàskÕttër nàvïgàtïõn !!\]/,
        }),
      ).toBeVisible();
      expect(screen.queryByText(/webShell\.|issues\./)).not.toBeInTheDocument();
    } finally {
      taskotterConsoleFixture.localePreferences = originalPreferences;
    }
  });
});
