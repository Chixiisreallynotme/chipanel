#!/usr/bin/env node
/**
 * Design-token guard. Fails when:
 *  1. a `var(--token)` has no definition in the same file or in lib/styles/tokens.css;
 *  2. a `<style>` block (or a global CSS file) hardcodes a hex colour;
 *  3. a `:root` block redefines tokens outside tokens.css.
 *
 * tokens.css is the single source of truth (design.md §12.1). A deliberate
 * exception is annotated on the same line with `token-ok: <reason>`.
 */
import { readdir, readFile } from 'node:fs/promises';
import { join, relative } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('..', import.meta.url));
const src = join(root, 'src');
const tokensFile = join(src, 'lib', 'styles', 'tokens.css');

const DEF = /^\s*(--[a-zA-Z0-9-]+)\s*:/gm;
const LOCAL_DEF = /(?:^|[\s"'{;])(--[a-zA-Z0-9-]+)\s*:/gm;
const USE = /var\(\s*(--[a-zA-Z0-9-]+)/g;
const HEX = /#[0-9A-Fa-f]{3,8}\b/g;

const globalTokens = new Set();
for (const match of (await readFile(tokensFile, 'utf8')).matchAll(DEF)) {
	globalTokens.add(match[1]);
}

async function* walk(dir) {
	for (const entry of await readdir(dir, { withFileTypes: true })) {
		const full = join(dir, entry.name);
		if (entry.isDirectory()) yield* walk(full);
		else if (/\.(svelte|css)$/.test(entry.name)) yield full;
	}
}

/** @type {string[]} */
const errors = [];
/** @type {string[]} */
const warnings = [];

function lineAt(text, index) {
	return text.slice(0, index).split('\n').length;
}

for await (const file of walk(src)) {
	const rel = relative(root, file).replaceAll('\\', '/');
	if (rel === 'src/lib/styles/tokens.css') continue;

	const text = await readFile(file, 'utf8');
	const localTokens = new Set([...text.matchAll(LOCAL_DEF)].map((m) => m[1]));

	for (const match of text.matchAll(USE)) {
		const token = match[1];
		if (!globalTokens.has(token) && !localTokens.has(token)) {
			errors.push(`${rel}:${lineAt(text, match.index)} unknown token ${token}`);
		}
	}

	if (rel.endsWith('.css') && /:root\s*\{/.test(text)) {
		errors.push(`${rel}:1 redefines :root outside tokens.css`);
	}

	const styleStart = rel.endsWith('.svelte') ? text.indexOf('<style') : 0;
	if (styleStart === -1) continue;

	const style = text.slice(styleStart);
	const baseLine = lineAt(text, styleStart);
	for (const match of style.matchAll(HEX)) {
		const line = baseLine + lineAt(style, match.index) - 1;
		const lineText = text.split('\n')[line - 1] ?? '';
		if (lineText.includes('token-ok:')) continue;
		errors.push(`${rel}:${line} hardcoded colour ${match[0]}`);
	}
}

if (errors.length > 0) {
	console.error(`check:tokens — ${errors.length} issue(s)`);
	for (const error of errors.slice(0, 40)) console.error(`  ${error}`);
	if (errors.length > 40) console.error(`  … ${errors.length - 40} more`);
	process.exit(1);
}

if (warnings.length > 0) {
	for (const warning of warnings) console.warn(`  ${warning}`);
}
console.log(`check:tokens — OK (${globalTokens.size} tokens, 0 drift)`);
