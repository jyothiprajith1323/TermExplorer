const { test, expect } = require('@playwright/test');
const AxeBuilder = require('@axe-core/playwright').default;

const pages = ['/index.html', '/features.html', '/architecture.html', '/docs.html', '/roadmap.html'];
const viewports = [
  { name: 'small-phone', width: 320, height: 640 },
  { name: 'phone', width: 360, height: 800 },
  { name: 'phone-390', width: 390, height: 844 },
  { name: 'large-phone', width: 430, height: 932 },
  { name: 'small-tablet', width: 600, height: 900 },
  { name: 'tablet-portrait', width: 768, height: 1024 },
  { name: 'tablet-820', width: 820, height: 1180 },
  { name: 'phone-landscape', width: 844, height: 390 },
  { name: 'mobile-breakpoint', width: 860, height: 1000 },
  { name: 'desktop-breakpoint', width: 861, height: 1000 },
  { name: 'phone-wide-landscape', width: 932, height: 430 },
  { name: 'tablet-landscape', width: 1024, height: 768 },
  { name: 'large-tablet', width: 1100, height: 800 },
  { name: 'compact-laptop', width: 1200, height: 600 },
  { name: 'laptop', width: 1366, height: 768 },
  { name: 'large-laptop', width: 1600, height: 900 },
  { name: 'desktop', width: 1920, height: 1080 },
  { name: 'qhd', width: 2560, height: 1440 },
  { name: 'ultrawide', width: 3440, height: 1440 },
  { name: '4k', width: 3840, height: 2160 },
];

for (const route of pages) {
  for (const viewport of viewports) {
    test(`responsive ${route} at ${viewport.name} (${viewport.width}x${viewport.height})`, async ({ page }) => {
      await page.setViewportSize({ width: viewport.width, height: viewport.height });
      const response = await page.goto(route);
      expect(response && response.ok(), `failed to load ${route}`).toBeTruthy();
      await expect(page.locator('main')).toBeVisible();
      await expect(page.locator('h1')).toHaveCount(1);

      const dimensions = await page.evaluate(() => ({
        viewport: document.documentElement.clientWidth,
        document: document.documentElement.scrollWidth,
        body: document.body.scrollWidth,
        brokenImages: [...document.images].filter(img => !img.complete || img.naturalWidth === 0).map(img => img.src),
        oversizedImages: [...document.images].filter(img => img.getBoundingClientRect().width > document.documentElement.clientWidth + 1).map(img => img.src),
      }));
      expect(dimensions.document, `horizontal document overflow: ${JSON.stringify(dimensions)}`).toBeLessThanOrEqual(dimensions.viewport + 1);
      expect(dimensions.body, `horizontal body overflow: ${JSON.stringify(dimensions)}`).toBeLessThanOrEqual(dimensions.viewport + 1);
      expect(dimensions.brokenImages, `broken images: ${dimensions.brokenImages.join(', ')}`).toEqual([]);
      expect(dimensions.oversizedImages, `images exceed viewport: ${dimensions.oversizedImages.join(', ')}`).toEqual([]);
    });
  }
}

test('responsive navigation opens, closes and supports Escape on mobile', async ({ page }) => {
  await page.setViewportSize({ width: 360, height: 800 });
  await page.goto('/index.html');
  const toggle = page.locator('.menu-toggle');
  const nav = page.locator('#primary-nav');
  await expect(toggle).toBeVisible();
  await expect(toggle).toHaveAttribute('aria-expanded', 'false');
  await toggle.click();
  await expect(toggle).toHaveAttribute('aria-expanded', 'true');
  await expect(nav).toBeVisible();
  await page.keyboard.press('Escape');
  await expect(toggle).toHaveAttribute('aria-expanded', 'false');
  await expect(toggle).toBeFocused();
});

test('copy buttons expose a usable label and source text', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/docs.html');
  const buttons = page.locator('button[data-copy]');
  await expect(buttons.first()).toBeVisible();
  expect(await buttons.count()).toBeGreaterThanOrEqual(5);
  for (const button of await buttons.all()) {
    expect((await button.getAttribute('data-copy')) || '').not.toBe('');
    expect((await button.innerText()).trim()).not.toBe('');
  }
});

for (const route of pages) {
  for (const viewport of [
    { name: 'mobile', width: 390, height: 844 },
    { name: 'desktop', width: 1440, height: 900 },
  ]) {
    test(`accessibility ${route} at ${viewport.name} has no serious or critical axe violations`, async ({ page }) => {
      await page.setViewportSize({ width: viewport.width, height: viewport.height });
      await page.goto(route);
      const results = await new AxeBuilder({ page }).withTags(['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa', 'wcag22aa', 'best-practice']).analyze();
      const serious = results.violations.filter(v => ['serious', 'critical'].includes(v.impact));
      expect(serious, serious.map(v => `${v.id}: ${v.help} (${v.nodes.length} nodes)`).join('\n')).toEqual([]);
    });
  }
}

for (const dpr of [2, 3]) {
  test(`high-DPI layout at ${dpr}x`, async ({ browser }) => {
    const context = await browser.newContext({ viewport: { width: 390, height: 844 }, deviceScaleFactor: dpr });
    const page = await context.newPage();
    await page.goto('/index.html');
    const dims = await page.evaluate(() => ({ viewport: document.documentElement.clientWidth, document: document.documentElement.scrollWidth }));
    expect(dims.document, `high-DPI ${dpr}x viewport overflow: ${JSON.stringify(dims)}`).toBeLessThanOrEqual(dims.viewport + 1);
    await context.close();
  });
}
