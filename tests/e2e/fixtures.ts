import { test as base, expect } from "@playwright/test";
import fs from "node:fs/promises";
import path from "node:path";

const coverageEnabled = process.env.COVERAGE === "true";

export const test = base.extend({
  page: async ({ page }, use, testInfo) => {
    let coverageStarted = false;

    if (coverageEnabled) {
      await page.coverage.startJSCoverage({ resetOnNavigation: false });
      coverageStarted = true;
    }

    await use(page);

    if (!coverageEnabled || !coverageStarted) {
      return;
    }

    const coverage = await page.coverage.stopJSCoverage();
    const outputDir = path.join(process.cwd(), "tests/coverage/e2e/raw");
    await fs.mkdir(outputDir, { recursive: true });

    const rawName = `${testInfo.project.name}-${testInfo.titlePath().join(" ")}-${
      testInfo.workerIndex
    }`;
    const safeName = rawName.replace(/[^a-z0-9]+/gi, "_").replace(/^_+|_+$/g, "");
    const outputPath = path.join(outputDir, `${safeName}.json`);

    await fs.writeFile(outputPath, JSON.stringify(coverage));
  },
});

export { expect };
