import AxeBuilder from '@axe-core/playwright';
import { expect, test } from '@playwright/test';

test('home loads without console errors and passes serious accessibility checks', async ({ page }) => {
  const errors: string[] = [];
  page.on('console', (message) => { if (message.type() === 'error') errors.push(message.text()); });
  await page.goto('/');
  await expect(page.getByRole('heading', { level: 1 })).toHaveText(/Know your backup/);
  await expect(page.locator('main')).toHaveCount(1);
  await expect(page.locator('h1')).toHaveCount(1);
  await expect(page.getByRole('img')).toHaveJSProperty('complete', true);
  const results = await new AxeBuilder({ page }).analyze();
  expect(results.violations.filter((item) => ['serious', 'critical'].includes(item.impact ?? ''))).toEqual([]);
  expect(errors).toEqual([]);
});

test('keyboard reaches the skip link and recorded demo controls', async ({ page }) => {
  await page.goto('/');
  await page.keyboard.press('Tab');
  await expect(page.getByRole('link', { name: 'Skip to main content' })).toBeFocused();
  await page.getByRole('button', { name: 'Play demo' }).focus();
  await page.keyboard.press('Enter');
  await expect(page.getByRole('button', { name: 'Pause demo' })).toBeVisible();
});

test('offline state explains what remains available', async ({ page, context }) => {
  await page.goto('/');
  await context.setOffline(true);
  await page.evaluate(() => window.dispatchEvent(new Event('offline')));
  await expect(page.getByText(/free CLI instructions still work/)).toBeVisible();
});

test('invalid returned license is stripped and does not gate free content', async ({ page }) => {
  await page.route('**/api/v1/products/restore-rehearsal-card/verify?license=bad-token', (route) => route.fulfill({ json: { valid: false, reason: 'invalid', expires_at: null } }));
  await page.goto('/?license=bad-token#operator-pack');
  await expect(page).toHaveURL(/\/#operator-pack$/);
  await expect(page.getByText(/License no longer active/)).toBeVisible();
  await expect(page.getByRole('link', { name: 'Run your first drill' })).toBeVisible();
});

test('legal pages have one heading and a main landmark', async ({ page }) => {
  for (const path of ['/privacy/', '/terms/']) {
    await page.goto(path);
    await expect(page.locator('h1')).toHaveCount(1);
    await expect(page.locator('main')).toHaveCount(1);
  }
});

test('390px layout has no horizontal overflow', async ({ page }, testInfo) => {
  test.skip(testInfo.project.name !== 'mobile', 'mobile project only');
  await page.goto('/');
  const sizes = await page.evaluate(() => ({ scroll: document.documentElement.scrollWidth, client: document.documentElement.clientWidth }));
  expect(sizes.scroll).toBeLessThanOrEqual(sizes.client);
  await expect(page.getByRole('heading', { level: 1 })).toBeVisible();
});
