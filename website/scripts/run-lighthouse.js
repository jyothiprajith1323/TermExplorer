'use strict';

const fs = require('node:fs');
const path = require('node:path');
const { spawnSync } = require('node:child_process');
const { chromium } = require('playwright');

const root = path.resolve(__dirname, '..');
const out = path.join(root, 'lighthouse-reports');
fs.mkdirSync(out, { recursive: true });
const chromePath = chromium.executablePath();
const routes = [
  { route: '/', name: 'home-mobile' },
  { route: '/features.html', name: 'features-mobile' },
  { route: '/architecture.html', name: 'architecture-mobile' },
  { route: '/docs.html', name: 'docs-mobile' },
  { route: '/roadmap.html', name: 'roadmap-mobile' },
];

for (const item of routes) {
  const report = path.join(out, `${item.name}.json`);
  const result = spawnSync(process.execPath, [
    path.join(root, 'node_modules/lighthouse/cli/index.js'),
    `http://127.0.0.1:${process.env.WEBSITE_TEST_PORT || 4173}${item.route}`,
    '--chrome-path', chromePath,
    '--chrome-flags=--headless --no-sandbox',
    '--only-categories=performance,accessibility,best-practices,seo',
    '--form-factor=mobile',
    '--output=json',
    `--output-path=${report}`,
    '--quiet',
  ], { cwd: root, stdio: 'inherit', env: process.env });
  if (result.status !== 0) process.exit(result.status || 1);
}

// A separate desktop pass checks the layout and desktop performance profile.
const desktopReport = path.join(out, 'home-desktop.json');
const desktop = spawnSync(process.execPath, [
  path.join(root, 'node_modules/lighthouse/cli/index.js'),
  `http://127.0.0.1:${process.env.WEBSITE_TEST_PORT || 4173}/`,
  '--chrome-path', chromePath,
  '--chrome-flags=--headless --no-sandbox',
  '--only-categories=performance,accessibility,best-practices,seo',
  '--form-factor=desktop',
  '--output=json',
  `--output-path=${desktopReport}`,
  '--quiet',
], { cwd: root, stdio: 'inherit', env: process.env });
if (desktop.status !== 0) process.exit(desktop.status || 1);

const thresholds = { performance: 0.80, accessibility: 0.95, 'best-practices': 0.90, seo: 0.90 };
const failures = [];
for (const file of fs.readdirSync(out).filter(name => name.endsWith('.json'))) {
  const data = JSON.parse(fs.readFileSync(path.join(out, file), 'utf8'));
  for (const [category, minimum] of Object.entries(thresholds)) {
    const score = data.categories?.[category]?.score;
    if (typeof score !== 'number' || score < minimum) {
      failures.push(`${file}: ${category}=${score} (minimum ${minimum})`);
    }
  }
}
if (failures.length) {
  console.error('Lighthouse quality gates failed:\n' + failures.map(x => ` - ${x}`).join('\n'));
  process.exit(1);
}
console.log(`Lighthouse gates passed for ${fs.readdirSync(out).filter(x => x.endsWith('.json')).length} reports.`);
