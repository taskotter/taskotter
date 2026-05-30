import { readFile, readdir } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

const root = process.cwd();
const frontendRoots = [
  "src",
  "apps/web",
  "apps/desktop",
  "apps/mobile",
  "packages/ui",
  "packages/workflow",
];
const allowedContractImports = [
  "packages/api-client",
  "packages/schemas",
  "@taskotter/api-client",
  "@taskotter/schemas",
];
const forbiddenImportPatterns = [
  /(?:^|['"])services\/api(?:\/|['"])/,
  /(?:^|['"])services\/runner(?:\/|['"])/,
  /(?:^|['"])services\/gateway(?:\/|['"])/,
  /\.\.\/(?:\.\.\/)*services\/api(?:\/|['"])/,
  /\.\.\/(?:\.\.\/)*services\/runner(?:\/|['"])/,
  /\.\.\/(?:\.\.\/)*services\/gateway(?:\/|['"])/,
];
const sourceExtensions = new Set([
  ".ts",
  ".tsx",
  ".js",
  ".jsx",
  ".mjs",
  ".cjs",
]);
const importPattern =
  /(?:import|export)\s+(?:type\s+)?(?:[^'"]*from\s+)?['"]([^'"]+)['"]|require\(['"]([^'"]+)['"]\)/g;

async function pathExists(relativePath) {
  try {
    await readdir(path.join(root, relativePath));
    return true;
  } catch {
    return false;
  }
}

async function walk(relativePath) {
  const absolute = path.join(root, relativePath);
  const entries = await readdir(absolute, { withFileTypes: true });
  const files = [];
  for (const entry of entries) {
    if (
      entry.name === "node_modules" ||
      entry.name === "dist" ||
      entry.name === ".next"
    )
      continue;
    const child = path.join(relativePath, entry.name);
    if (entry.isDirectory()) {
      files.push(...(await walk(child)));
    } else if (sourceExtensions.has(path.extname(entry.name))) {
      files.push(child);
    }
  }
  return files;
}

function isAllowedContractImport(specifier) {
  return allowedContractImports.some(
    (allowed) => specifier === allowed || specifier.startsWith(`${allowed}/`),
  );
}

function validateImport(fileName, specifier) {
  if (isAllowedContractImport(specifier)) return;
  for (const pattern of forbiddenImportPatterns) {
    if (pattern.test(specifier)) {
      throw new Error(
        `${fileName} imports forbidden backend/runtime internals: ${specifier}`,
      );
    }
  }
}

async function main() {
  const checkedFiles = [];
  for (const frontendRoot of frontendRoots) {
    if (!(await pathExists(frontendRoot))) continue;
    checkedFiles.push(...(await walk(frontendRoot)));
  }

  for (const fileName of checkedFiles) {
    const source = await readFile(path.join(root, fileName), "utf8");
    for (const match of source.matchAll(importPattern)) {
      validateImport(fileName, match[1] ?? match[2]);
    }
  }

  console.log(
    `Module boundary check passed (${checkedFiles.length} frontend files scanned).`,
  );
}

main().catch((error) => {
  console.error(error.message);
  process.exit(1);
});
