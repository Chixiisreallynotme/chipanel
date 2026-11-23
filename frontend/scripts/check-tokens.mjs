#!/usr/bin/env node
/**
 * Design-system guard (Design Council VI). Fails when:
 *  1. a `var(--token)` has no definition in the same file or in lib/styles/tokens.css;
 *  2. a `<style>` block (or a global CSS file) hardcodes a hex colour;
 *  3. a `:root` block redefines tokens outside tokens.css;
 *  4. a component redeclares a canonical global class and is not registered in
 *     ui-baseline.json (i.e. a NEW duplicate, the exact drift this council kills);
 *  5. a component declares its own `@keyframes spin` (one global animation only);
 *  6. a route has no `<title>`, uses a non-uniform suffix, or uses an em dash;
 *  7. app.html does not declare `lang="fr"`.
 *
 * tokens.css is the single source of truth (design.md §12.1). A deliberate
 * exception is annotated on the same line with `token-ok: <reason>`.
 * The baseline is a snapshot of the local variants we consciously keep; it is
 * regenerated only by an explicit decision, never silently on a normal run.
 */
import { readdir, readFile } from 'node:fs/promises';
import { join, relative } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('..', import.meta.url));
const src = join(root, 'src');
const tokensFile = join(src, 'lib', 'styles', 'tokens.css');
const appCss = join(src, 'app.css');
const baselineFile = join(root, 'scripts', 'ui-baseline.json');

const DEF = /^\s*(--[a-zA-Z0-9-]+)\s*:/gm;
const LOCAL_DEF = /(?:^|[\s"'{;])(--[a-zA-Z0-9-]+)\s*:/gm;
const USE = /var\(\s*(--[a-zA-Z0-9-]+)/g;
const HEX = /#[0-9A-Fa-f]{3,8}\b/g;

const globalTokens = new Set();
for (const match of (await readFile(tokensFile, 'utf8')).matchAll(DEF)) {
	globalTokens.add(match[1]);
}

/** Standalone simple class selectors at the start of a rule in a global file. */
const canonical = new Set();
for (const file of [appCss, tokensFile]) {
	const text = await readFile(file, 'utf8');
	for (const m of text.matchAll(/^\s*\.([a-zA-Z][a-zA-Z0-9_-]*)\s*(?:,|\{)/gm)) {
		canonical.add(m[1]);
	}
}

let baseline = { canonical: [], local: {} };
try {
	baseline = JSON.parse(await readFile(baselineFile, 'utf8'));
} catch {
	console.error('check:tokens — ui-baseline.json missing or invalid');
	process.exit(1);
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
const notes = [];

function lineAt(text, index) {
	return text.slice(0, index).split('\n').length;
}

const localDefs = new Map();

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

	if (!rel.endsWith('.svelte')) continue;

	for (const cls of canonical) {
		const re = new RegExp('(?:^|\\n)\\s*(?:\\.|[^\\n]*\\s)\\.' + cls + '\\b[^{}]*\\{', 'm');
		if (!re.test(style)) continue;
		// Undeclared in the baseline => known is empty => any local copy fails below.
		if (!localDefs.has(cls)) localDefs.set(cls, []);
		// ui-baseline.json stores paths relative to src/, not to frontend/.
		localDefs.get(cls).push(rel.replace(/^src\//, ''));
	}

	if (/@keyframes\s+spin\b/.test(style)) {
		errors.push(`${rel}: declares a local @keyframes spin (use the global one in app.css)`);
	}
}

for (const [cls, files] of localDefs) {
	const known = new Set(baseline.local[cls] ?? []);
	for (const file of files) {
		if (!known.has(file)) {
			errors.push(`${file}: NEW duplicate of canonical .${cls} (not in ui-baseline.json)`);
		}
	}
}

// Route titles: one convention, one suffix, no em dash.
const routeFiles = [];
async function collectRoutes(dir) {
	for (const entry of await readdir(dir, { withFileTypes: true })) {
		const full = join(dir, entry.name);
		if (entry.isDirectory()) await collectRoutes(full);
		else if (entry.name === '+page.svelte') routeFiles.push(full);
	}
}
await collectRoutes(join(src, 'routes'));
for (const file of routeFiles) {
	const rel = relative(root, file).replaceAll('\\', '/');
	const text = await readFile(file, 'utf8');
	const match = text.match(/<title>([^<]*)<\/title>/);
	if (!match) {
		errors.push(`${rel}: missing <title>`);
		continue;
	}
	const title = match[1].trim();
	if (!/^.+ \| ChiPanel$/.test(title)) {
		errors.push(`${rel}: <title> must end with " | ChiPanel" (got "${title}")`);
	}
	if (title.includes('—')) {
		errors.push(`${rel}: <title> contains an em dash (R-02)`);
	}
}

const appHtml = await readFile(join(src, 'app.html'), 'utf8');
if (!/<html lang="fr">/.test(appHtml)) {
	errors.push('src/app.html: <html lang="fr"> is required for the French UI');
}

if (errors.length > 0) {
	console.error(`check:tokens — ${errors.length} issue(s)`);
	for (const error of errors.slice(0, 40)) console.error(`  ${error}`);
	if (errors.length > 40) console.error(`  … ${errors.length - 40} more`);
	process.exit(1);
}

const reduction = Object.entries(baseline.local).filter(([cls, files]) => {
	const now = localDefs.get(cls) ?? [];
	return now.length < files.length;
});
for (const [cls, files] of reduction) {
	notes.push(`.${cls}: ${files.length - (localDefs.get(cls) ?? []).length} local copy/copies removed`);
}
for (const note of notes) console.log(`  note: ${note}`);
console.log(`check:tokens — OK (${globalTokens.size} tokens, ${canonical.size} canonical selectors, 0 drift)`);
