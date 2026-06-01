import { render, screen, within } from "@testing-library/react";
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

    const composer = screen.getByRole("form", { name: /reply composer/i });
    expect(within(composer).getByLabelText("Reply")).toBeVisible();
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
