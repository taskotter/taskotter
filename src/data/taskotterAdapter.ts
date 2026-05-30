import type { ConsoleData, TaskOtterDataAdapter } from "./contracts";
import { taskotterConsoleFixture } from "./taskotterFixtures";

export class FixtureTaskOtterDataAdapter implements TaskOtterDataAdapter {
  async getConsoleData(): Promise<ConsoleData> {
    return structuredClone(taskotterConsoleFixture);
  }
}

export const taskOtterDataAdapter = new FixtureTaskOtterDataAdapter();
