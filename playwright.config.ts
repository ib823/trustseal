/**
 * Playwright E2E Test Configuration
 *
 * Tests the VaultPass web applications for Phase 2 hardening.
 *
 * Run all tests:
 *   pnpm exec playwright test
 *
 * Run specific suite:
 *   pnpm exec playwright test tests/e2e/admin-portal.spec.ts
 */

import { defineConfig, devices } from '@playwright/test';

const isCI = !!process.env.CI;

export default defineConfig({
  testDir: './tests/e2e',
  fullyParallel: true,
  forbidOnly: isCI,
  retries: isCI ? 2 : 0,
  workers: isCI ? 1 : undefined,
  reporter: [
    ['html', { open: 'never' }],
    ['list'],
  ],
  use: {
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
    video: 'on-first-retry',
  },

  // Performance target: verification landing page < 1.5s FMP
  expect: {
    timeout: 10000,
  },
  timeout: 30000,

  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
    {
      name: 'mobile-chrome',
      use: { ...devices['Pixel 5'] },
    },
    {
      name: 'tablet',
      use: {
        ...devices['iPad Pro 11'],
        // Guard tablet is landscape-optimized
        viewport: { width: 1194, height: 834 },
      },
    },
  ],

  webServer: [
    {
      command: 'cargo run -p platform-api',
      url: 'http://localhost:3000/health',
      reuseExistingServer: !isCI,
      timeout: 120000,
      env: {
        RUST_LOG: 'info',
      },
    },
    {
      command: 'pnpm --filter @sahi/admin-portal dev',
      url: 'http://localhost:3001',
      reuseExistingServer: !isCI,
      timeout: 60000,
    },
    {
      command: 'pnpm --filter @sahi/guard-tablet dev',
      url: 'http://localhost:3002',
      reuseExistingServer: !isCI,
      timeout: 60000,
    },
  ],
});
