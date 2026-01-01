import fs from "node:fs/promises";
import path from "node:path";

const projectRoot = process.cwd();
const rawDir = path.join(projectRoot, "tests/coverage/e2e/raw");
const reportDir = path.join(projectRoot, "tests/coverage/e2e/coverage-report");

const exists = async (filePath) => {
  try {
    await fs.access(filePath);
    return true;
  } catch {
    return false;
  }
};

const shouldIncludeEntry = (url) => {
  if (!url) return false;
  if (!url.includes("/static/")) return false;
  return url.endsWith(".js") || url.endsWith(".mjs");
};

const mergeRanges = (ranges) => {
  if (!ranges.length) return [];
  const sorted = [...ranges].sort((a, b) => a[0] - b[0]);
  const merged = [sorted[0]];
  for (let i = 1; i < sorted.length; i += 1) {
    const [start, end] = sorted[i];
    const last = merged[merged.length - 1];
    if (start <= last[1]) {
      last[1] = Math.max(last[1], end);
    } else {
      merged.push([start, end]);
    }
  }
  return merged;
};

const sumRanges = (ranges) =>
  ranges.reduce((sum, [start, end]) => sum + Math.max(0, end - start), 0);

const collectCoverage = (coverageEntries) => {
  let totalBytes = 0;
  let coveredBytes = 0;
  let totalFunctions = 0;
  let coveredFunctions = 0;

  for (const entry of coverageEntries) {
    if (!shouldIncludeEntry(entry.url)) {
      continue;
    }

    const allRanges = [];
    const hitRanges = [];

    for (const fn of entry.functions || []) {
      totalFunctions += 1;
      const hasHit = (fn.ranges || []).some((range) => range.count > 0);
      if (hasHit) {
        coveredFunctions += 1;
      }
      for (const range of fn.ranges || []) {
        allRanges.push([range.startOffset, range.endOffset]);
        if (range.count > 0) {
          hitRanges.push([range.startOffset, range.endOffset]);
        }
      }
    }

    totalBytes += sumRanges(mergeRanges(allRanges));
    coveredBytes += sumRanges(mergeRanges(hitRanges));
  }

  return { totalBytes, coveredBytes, totalFunctions, coveredFunctions };
};

if (!(await exists(rawDir))) {
  console.error(`E2E coverage raw directory missing: ${rawDir}`);
  process.exit(1);
}

const rawFiles = (await fs.readdir(rawDir)).filter((file) => file.endsWith(".json"));
if (rawFiles.length === 0) {
  console.error(`No E2E coverage data found in ${rawDir}`);
  process.exit(1);
}

let totals = {
  totalBytes: 0,
  coveredBytes: 0,
  totalFunctions: 0,
  coveredFunctions: 0,
};
let totalEntries = 0;
let filteredEntries = 0;

for (const file of rawFiles) {
  const raw = await fs.readFile(path.join(rawDir, file), "utf8");
  const entries = JSON.parse(raw);
  totalEntries += entries.length;
  filteredEntries += entries.filter((entry) => shouldIncludeEntry(entry.url)).length;
  const snapshot = collectCoverage(entries);
  totals.totalBytes += snapshot.totalBytes;
  totals.coveredBytes += snapshot.coveredBytes;
  totals.totalFunctions += snapshot.totalFunctions;
  totals.coveredFunctions += snapshot.coveredFunctions;
}

const linePct =
  totals.totalBytes === 0
    ? 0
    : (totals.coveredBytes / totals.totalBytes) * 100;
const functionPct =
  totals.totalFunctions === 0
    ? 0
    : (totals.coveredFunctions / totals.totalFunctions) * 100;

await fs.mkdir(reportDir, { recursive: true });

const summary = {
  generatedAt: new Date().toISOString(),
  sources: {
    totalEntries,
    filteredEntries,
  },
  coverage: {
    lines: { pct: Number(linePct.toFixed(2)) },
    functions: { pct: Number(functionPct.toFixed(2)) },
  },
  totals: {
    bytes: totals,
  },
};

await fs.writeFile(
  path.join(reportDir, "coverage-summary.json"),
  JSON.stringify(summary, null, 2),
);

const html = `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>E2E Coverage Summary</title>
  <style>
    body {
      font-family: "Segoe UI", Tahoma, Geneva, Verdana, sans-serif;
      margin: 40px;
      color: #1f2937;
    }
    .card {
      max-width: 640px;
      padding: 24px;
      border-radius: 16px;
      background: #f8fafc;
      box-shadow: 0 8px 24px rgba(15, 23, 42, 0.08);
    }
    h1 {
      margin: 0 0 12px;
    }
    .metric {
      display: flex;
      justify-content: space-between;
      margin: 12px 0;
    }
    .label {
      font-weight: 600;
    }
    .value {
      font-variant-numeric: tabular-nums;
    }
    .meta {
      margin-top: 16px;
      font-size: 0.9rem;
      color: #6b7280;
    }
  </style>
</head>
<body>
  <div class="card">
    <h1>E2E Coverage Summary</h1>
    <div class="metric">
      <span class="label">Line (byte) coverage</span>
      <span class="value">${linePct.toFixed(2)}%</span>
    </div>
    <div class="metric">
      <span class="label">Function coverage</span>
      <span class="value">${functionPct.toFixed(2)}%</span>
    </div>
    <div class="meta">
      Generated at ${summary.generatedAt}<br />
      Scripts analyzed: ${filteredEntries} of ${totalEntries}
    </div>
  </div>
</body>
</html>`;

await fs.writeFile(path.join(reportDir, "index.html"), html);

console.log(`E2E coverage summary written to ${reportDir}`);
