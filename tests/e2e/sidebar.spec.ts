import { test, expect } from './fixtures';

test('sidebar history and session switching', async ({ page }) => {
  // 1. Setup
  page.on('console', msg => console.log(`BROWSER LOG: ${msg.text()}`));
  const dbReadyPromise = page.waitForEvent('console', msg => msg.text().includes('PGlite database initialized successfully'));
  await page.goto('/');
  await dbReadyPromise;

  // 2. Create first conversation
  await page.locator('textarea[name="message"]').fill('Session One');
  await page.keyboard.press('Enter');
  await page.waitForTimeout(1000); 
  // Wait for auto-naming to likely fire or title to settle
  await expect(page.locator('.assistant-message')).toBeVisible();

  // 3. New Chat
  const newChatBtn = page.locator('button', { hasText: 'New Chat' }); // Adjust selector to match actual UI
  if (await newChatBtn.isVisible()) {
      await newChatBtn.click();
  } else {
      // Fallback if icon based
      // Looking for a typical "New" icon or button
      await page.locator('.new-chat-icon, [aria-label="New Chat"]').first().click();
  }

  // 4. Create second conversation
  await page.locator('textarea[name="message"]').fill('Session Two');
  await page.keyboard.press('Enter');
  await expect(page.locator('.assistant-message')).toBeVisible();

  // 5. Verify Sidebar entries
  const sidebarItems = page.locator('conversation-sidebar .conversation-item'); // Adjust selector
  // We expect at least these 2 sessions
  // Note: Sidebar might load async from PGlite
  await expect(sidebarItems.first()).toBeVisible();
  const count = await sidebarItems.count();
  expect(count).toBeGreaterThanOrEqual(2);

  // 6. Switch back to first session
  // Assuming the list is ordered by recency, the second one in the list (or first if we're technically on the second)
  // We just click a different one.
  const secondItem = sidebarItems.nth(1);
  await secondItem.click();

  // 7. Verification of switch
  // message list should contain "Session One"
  await expect(page.locator('.user-message').filter({ hasText: 'Session One' })).toBeVisible();
});
