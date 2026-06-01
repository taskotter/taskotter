import { readFileSync, readdirSync } from "node:fs";
import { extname, join, relative } from "node:path";

const root = new URL("..", import.meta.url).pathname;
const sourceRoot = join(root, "src");
const checkedExtensions = new Set([".tsx"]);
const propPattern =
  /\b(aria-label|placeholder|title|alt)\s*=\s*["']([^"'{][^"']*[A-Za-z][^"']*)["']/g;
const jsxTextPattern = />\s*([^<>{}]*[A-Za-z][^<>{}]*)\s*</g;

const ignoredFiles = [/\.test\.tsx$/];
const ignoredText = new Set(["TO"]);

function listFiles(dir) {
  return readdirSync(dir, { withFileTypes: true }).flatMap((entry) => {
    const path = join(dir, entry.name);
    if (entry.isDirectory()) return listFiles(path);
    if (!checkedExtensions.has(extname(entry.name))) return [];
    if (ignoredFiles.some((pattern) => pattern.test(path))) return [];
    return [path];
  });
}

const violations = [];

for (const file of listFiles(sourceRoot)) {
  const source = readFileSync(file, "utf8");
  const relativePath = relative(root, file);

  for (const match of source.matchAll(propPattern)) {
    violations.push(`${relativePath}: localize ${match[1]}="${match[2]}"`);
  }

  for (const match of source.matchAll(jsxTextPattern)) {
    const text = match[1].replace(/\s+/g, " ").trim();
    if (!text || ignoredText.has(text)) continue;
    if (/^[A-Za-z_$][\w$]*(\.|\(|\[)/.test(text)) continue;
    violations.push(`${relativePath}: localize JSX text "${text}"`);
  }
}

if (violations.length > 0) {
  console.error("Hard-coded user-visible strings found:");
  for (const violation of violations) console.error(`- ${violation}`);
  process.exit(1);
}

console.log("No hard-coded JSX text or visible string props found.");
