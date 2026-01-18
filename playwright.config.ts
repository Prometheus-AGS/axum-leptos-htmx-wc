import { defineConfig, devices } from '@playwright/test';

const useDockerApp = process.env.USE_DOCKER_APP === 'true';

export default defineConfig({
  testDir: './tests/e2e',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: [
    ['html'],
    ['json', { outputFile: 'test-results/results.json' }]
  ],
  use: {
    baseURL: 'http://127.0.0.1:3001',
    trace: 'on-first-retry',
    // Enable V8 coverage collection
    ...(process.env.COVERAGE === 'true' && {
      contextOptions: {
        // Enable code coverage collection
        recordVideo: { mode: 'retain-on-failure' },
      },
    }),
  },
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
  ],
  webServer: useDockerApp
    ? undefined
    : {
        command: 'cargo run',
        url: 'http://127.0.0.1:3001',
        reuseExistingServer: !process.env.CI,
        timeout: 120 * 1000,
      },
});
