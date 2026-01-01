import { test, expect } from './fixtures';

test('verify tool execution and rendering', async ({ page }) => {
  // 1. Setup logs
  page.on('console', msg => console.log(`BROWSER LOG: ${msg.text()}`));
  
  // 2. Load Page and wait for DB
  const dbReadyPromise = page.waitForEvent('console', msg => msg.text().includes('PGlite database initialized successfully'));
  await page.goto('/');
  await expect(page).toHaveTitle(/Prometheus/);
  await dbReadyPromise;

  // 3. Ask a question that requires a tool (Time)
  const input = page.locator('textarea[name="message"]');
  await input.fill('What time is it right now?');
  // Use a mock response pattern if we were mocking, but here we assume the real server has the Time tool enabled.
  await page.keyboard.press('Enter');

  // 4. Verify user message
  await expect(page.locator('.user-message').filter({ hasText: 'What time is it right now?' })).toBeVisible();

  // 5. Expect a tool use block
  // The UI should render a tool execution block
  const toolBlock = page.locator('tool-call'); 
  // We wait for it to appear
  await expect(toolBlock).toBeVisible({ timeout: 15000 });

  // 6. Verify the tool name is visible (assuming generic tool rendering)
  // Adjust selector based on actual generic tool rendering
  await expect(toolBlock).toContainText('get_current_time');

  // 7. Verify final answer also appears
  const assistantMsg = page.locator('.assistant-message').last();
  await expect(assistantMsg).toBeVisible({ timeout: 30000 });
  await expect(assistantMsg).not.toBeEmpty();
});
