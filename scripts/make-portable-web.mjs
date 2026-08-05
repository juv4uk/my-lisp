import { readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import process from 'node:process';

const [, , inputPath, outputPath] = process.argv;

if (!inputPath || !outputPath) {
  throw new Error('Usage: node scripts/make-portable-web.mjs <input.html> <output.html>');
}

// EN: Every visual asset used by the page is converted to a data URI. The
// result can be moved, renamed, or opened from file:// without a companion
// directory.
// UK: Кожен візуальний ресурс сторінки перетворюється на data URI. Результат
// можна переносити, перейменовувати й відкривати через file:// без додаткової
// папки.
// DE: Alle visuellen Seitendateien werden in Data-URIs umgewandelt. Dadurch
// kann die HTML-Datei ohne Begleitordner verschoben, umbenannt und über
// file:// geöffnet werden.
const embeddedAssets = [
  { reference: './brand-icon.svg', file: 'brand-icon.svg', mime: 'image/svg+xml' },
  { reference: './favicon.png', file: 'favicon.png', mime: 'image/png' },
  { reference: './apple-touch-icon.png', file: 'apple-touch-icon.png', mime: 'image/png' },
];

let html = await readFile(inputPath, 'utf8');

for (const asset of embeddedAssets) {
  const source = await readFile(path.join('static', asset.file));
  const dataUri = `data:${asset.mime};base64,${source.toString('base64')}`;
  html = html.replaceAll(asset.reference, dataUri);
}

// EN/UK/DE: Fail the release instead of publishing a file that still depends
// on local JavaScript, CSS, icons, or SvelteKit runtime files.
const forbiddenReferences = [
  /<script[^>]+src=/i,
  /<link[^>]+rel=["']modulepreload["']/i,
  /(?:href|src)=["']\.?\/?(?:_app|brand-icon|favicon|apple-touch-icon)/i,
];

if (forbiddenReferences.some((pattern) => pattern.test(html))) {
  throw new Error('Portable Web HTML still contains an external local asset reference');
}

await writeFile(outputPath, html, 'utf8');
console.log(`Standalone Web HTML created: ${outputPath} (${Buffer.byteLength(html)} bytes)`);
