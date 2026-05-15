#!/usr/bin/env node

/**
 * Smoke test the curated docs build.
 */

import fs from 'fs';
import path from 'path';

const rootDir = process.cwd();
const distDir = path.join(rootDir, 'dist', 'docs-site');
const config = JSON.parse(fs.readFileSync(path.join(rootDir, 'docs.json'), 'utf8'));

function collectPages(node, pages = []) {
  if (Array.isArray(node)) {
    for (const item of node) collectPages(item, pages);
    return pages;
  }

  if (!node || typeof node !== 'object') return pages;

  if (Array.isArray(node.pages)) {
    for (const page of node.pages) {
      if (typeof page === 'string') pages.push(page);
      else collectPages(page, pages);
    }
  }

  for (const value of Object.values(node)) {
    if (value !== node.pages) collectPages(value, pages);
  }

  return pages;
}

function outputPathForPage(page) {
  const normalized = page.replace(/^\/+/, '').replace(/\.md$/, '');
  if (normalized === 'index') return path.join(distDir, 'index.html');
  return path.join(distDir, normalized, 'index.html');
}

function pageUrl(page) {
  const normalized = page.replace(/^\/+/, '').replace(/\.md$/, '').replace(/\/index$/, '');
  return normalized === 'index' ? '/' : `/${normalized}`;
}

function assertFile(file, label = file) {
  if (!fs.existsSync(file)) {
    throw new Error(`${label} not found`);
  }
}

console.log('Running docs smoke tests...');

const pages = [...new Set(collectPages(config.navigation ?? {}))];
const publicUrls = new Set(pages.map(pageUrl));

for (const page of pages) {
  assertFile(outputPathForPage(page), page);
}

assertFile(path.join(distDir, 'search-index.json'), 'search-index.json');
assertFile(path.join(distDir, 'manifest.json'), 'manifest.json');
assertFile(path.join(distDir, 'style.css'), 'style.css');
assertFile(path.join(distDir, 'assets', 'opencoven-logo.svg'), 'opencoven-logo.svg');

const searchIndex = JSON.parse(fs.readFileSync(path.join(distDir, 'search-index.json'), 'utf8'));
if (searchIndex.length !== pages.length) {
  throw new Error(`search-index.json has ${searchIndex.length} entries, expected ${pages.length}`);
}

const stalePaths = [
  path.join(distDir, 'docs', 'guides', 'create-agent', 'index.html'),
  path.join(distDir, 'guides', 'create-agent', 'index.html'),
  path.join(distDir, 'core', 'agents', 'index', 'index.html'),
  path.join(distDir, 'resources', 'examples', 'basic-workflow', 'index.html')
];

for (const stalePath of stalePaths) {
  if (fs.existsSync(stalePath)) {
    throw new Error(`stale unrelated page was built: ${path.relative(distDir, stalePath)}`);
  }
}

for (const page of pages) {
  const file = path.join(rootDir, `${page.replace(/^\/+/, '').replace(/\.md$/, '')}.md`);
  const markdown = fs.readFileSync(file, 'utf8');
  const linkPattern = /(?:href="|\]\()([^")]+)(?:"|\))/g;
  let match;

  while ((match = linkPattern.exec(markdown))) {
    const rawUrl = match[1];
    if (!rawUrl.startsWith('/') || rawUrl.startsWith('//')) continue;

    const url = rawUrl.split('#')[0].replace(/\/$/, '') || '/';
    if (!publicUrls.has(url)) {
      throw new Error(`${page} links to non-public page: ${rawUrl}`);
    }
  }
}

console.log(`Smoke passed for ${pages.length} public pages`);
