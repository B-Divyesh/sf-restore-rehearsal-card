import AxeBuilder from '@axe-core/playwright';
import { expect, test } from '@playwright/test';

test('home loads without console errors and passes serious accessibility checks', async ({ page }) => {
  const errors: string[] = [];
  page.on('console', (message) => { if (message.type() === 'error') errors.push(message.text()); });
  await page.goto('/');
  await expect(page.getByRole('heading', { level: 1 })).toHaveText(/Test your backup/);
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
  await expect(page.getByRole('link', { name: 'Try it with sample data' })).toBeVisible();
});

test('secondary routes have one heading, a main landmark, and no serious accessibility issues', async ({ page }) => {
  for (const path of ['/demo/', '/privacy/', '/terms/', '/404.html']) {
    await page.goto(path);
    await expect(page.locator('h1')).toHaveCount(1);
    await expect(page.locator('main')).toHaveCount(1);
    const results = await new AxeBuilder({ page }).analyze();
    expect(results.violations.filter((item) => ['serious', 'critical'].includes(item.impact ?? ''))).toEqual([]);
  }
});

test('390px layout has no horizontal overflow', async ({ page }, testInfo) => {
  test.skip(testInfo.project.name !== 'mobile', 'mobile project only');
  await page.goto('/');
  const sizes = await page.evaluate(() => ({ scroll: document.documentElement.scrollWidth, client: document.documentElement.clientWidth }));
  expect(sizes.scroll).toBeLessThanOrEqual(sizes.client);
  await expect(page.getByRole('heading', { level: 1 })).toBeVisible();
});

test('@claim:sample-demo-route opens the isolated sample entry and reset starts fresh', async ({ page }) => {
  await page.goto('/demo/');
  await expect(page.getByRole('heading', { level: 1 })).toHaveText(/Run a sample restore drill/);
  await expect(page.getByText(/Demo — sample data, nothing is saved/)).toBeVisible();
  await page.getByRole('button', { name: 'Reset demo' }).click();
  await expect(page.locator('#demo-state')).toContainText(/Demo reset/);
  await expect(page.getByRole('link', { name: 'Start for real' })).toHaveAttribute('href', '/#install');
});

test('@claim:site-no-analytics makes no third-party request during the demo flow', async ({ page }) => {
  const origins = new Set<string>();
  page.on('request', (request) => origins.add(new URL(request.url()).origin));
  await page.goto('/demo/');
  await page.getByRole('button', { name: 'Reset demo' }).click();
  expect([...origins]).toEqual(['http://127.0.0.1:4173']);
});

test('@claim:production-checkout uses the production Sociobot endpoint', async ({ page }) => {
  await page.goto('/');
  await expect(page.locator('#buy-link')).toHaveAttribute(
    'href',
    'https://api.sociobot.in/api/v1/products/restore-rehearsal-card/checkout',
  );
});

test('every product route has canonical and social metadata', async ({ page }) => {
  for (const path of ['/', '/demo/', '/privacy/', '/terms/']) {
    await page.goto(path);
    await expect(page.locator('link[rel="canonical"]')).toHaveCount(1);
    await expect(page.locator('meta[property="og:image"]')).toHaveCount(1);
    await expect(page.locator('meta[name="twitter:card"]')).toHaveAttribute('content', 'summary_large_image');
  }
});
