import AxeBuilder from '@axe-core/playwright';
import { expect, test } from '@playwright/test';
import { readFile } from 'node:fs/promises';

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
  await expect(page.getByRole('heading', { level: 1 })).toHaveText(/Preview a sample restore drill/);
  await expect(page.getByText(/Demo — sample data, nothing is saved/)).toBeVisible();
  await expect(page.locator('#sample-output')).toContainText('PASS');
  await expect(page.locator('#sample-output')).toContainText('rrc-postgres-demo');
  await expect(page.locator('#sample-output')).toContainText('152 bytes');
  await expect(page.locator('#sample-output')).toContainText('f865…a721');
  await expect(page.locator('#sample-output')).toContainText('1 · expected 1');
  await expect(page.locator('#sample-output')).toContainText('Ed25519 valid');
  await page.evaluate(() => localStorage.setItem('real:operator-note', 'leave unchanged'));
  const before = await page.evaluate(() => JSON.stringify(localStorage));
  await page.getByRole('button', { name: 'Reset demo' }).click();
  await expect(page.locator('#demo-state')).toContainText(/Demo reset/);
  await expect(page.getByText(/Demo — sample data, nothing is saved/)).toBeVisible();
  await expect(page.locator('#sample-output')).toContainText('PASS');
  expect(await page.evaluate(() => JSON.stringify(localStorage))).toBe(before);
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
  await page.route('https://api.sociobot.in/api/v1/products/restore-rehearsal-card/checkout', (route) =>
    route.fulfill({ contentType: 'text/html', body: '<h1>Hosted checkout</h1>' }),
  );
  await page.goto('/');
  await page.locator('#buy-link').click();
  await expect(page).toHaveURL('https://api.sociobot.in/api/v1/products/restore-rehearsal-card/checkout');
  await expect(page.getByRole('heading', { name: 'Hosted checkout' })).toBeVisible();
});

test('@claim:operator-pack-offer provides the advertised paid files after verification', async ({ page }) => {
  await page.route('**/api/v1/products/restore-rehearsal-card/verify?license=valid-pack', (route) =>
    route.fulfill({ json: { valid: true, reason: 'ok', expires_at: null } }),
  );
  await page.goto('/?license=valid-pack#operator-pack');
  await expect(page.locator('.price')).toHaveText('$39 USD');
  await expect(page.getByText('One-time purchase · updates included')).toBeVisible();
  await expect(page.getByText(/Every CLI feature remains free/)).toBeVisible();
  await expect(page.getByText('License verified. The Operator Pack is available below.')).toBeVisible();

  const expected = [
    ['Download rehearsal runbook', 'restore-rehearsal-runbook.md', 'Execute rrc run with exact target confirmation'],
    ['Download RTO decision log', 'rto-decision-log.md', 'Objective | Observed | Result | Decision'],
    ['Download review & service profiles', 'restore-review-and-service-profiles.md', '## 6. Multi-service Compose stack'],
  ] as const;
  for (const [button, filename, content] of expected) {
    const [download] = await Promise.all([
      page.waitForEvent('download'),
      page.getByRole('button', { name: button }).click(),
    ]);
    expect(download.suggestedFilename()).toBe(filename);
    const body = await readFile(await download.path(), 'utf8');
    expect(body).toContain(content);
    if (filename === 'restore-review-and-service-profiles.md') {
      expect(body).toContain('# Restore review agenda');
      expect(body.match(/^## [1-6]\. /gm)).toHaveLength(6);
    }
  }
});

test('@claim:license-storage-cache stores, strips, and reuses a daily verdict', async ({ page }) => {
  let requests = 0;
  await page.route('**/api/v1/products/restore-rehearsal-card/verify?license=daily-token', (route) => {
    requests += 1;
    return route.fulfill({ json: { valid: true, reason: 'ok', expires_at: null } });
  });
  await page.goto('/?license=daily-token#operator-pack');
  await expect(page).toHaveURL(/\/#operator-pack$/);
  await expect(page.getByText('License verified. The Operator Pack is available below.')).toBeVisible();
  expect(await page.evaluate(() => localStorage.getItem('sb_license:restore-rehearsal-card'))).toBe('daily-token');
  await page.reload();
  await expect(page.getByText('License active. The Operator Pack is available below.')).toBeVisible();
  expect(requests).toBe(1);
});

test('every product route has canonical and social metadata', async ({ page }) => {
  for (const path of ['/', '/demo/', '/privacy/', '/terms/']) {
    await page.goto(path);
    await expect(page.locator('link[rel="canonical"]')).toHaveCount(1);
    await expect(page.locator('meta[property="og:image"]')).toHaveCount(1);
    await expect(page.locator('meta[name="twitter:card"]')).toHaveAttribute('content', 'summary_large_image');
  }
});
