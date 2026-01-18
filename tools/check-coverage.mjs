import fs from "node:fs/promises";
import path from "node:path";

const projectRoot = process.cwd();

const parseLcov = async (filePath) => {
  const text = await fs.readFile(filePath, "utf8");
  let linesFound = 0;
  let linesHit = 0;
  let functionsFound = 0;
  let functionsHit = 0;
  let branchesFound = 0;
  let branchesHit = 0;

  for (const line of text.split(/\r?\n/)) {
    if (line.startsWith("LF:")) {
      linesFound += Number(line.slice(3));
    } else if (line.startsWith("LH:")) {
      linesHit += Number(line.slice(3));
    } else if (line.startsWith("FNF:")) {
      functionsFound += Number(line.slice(4));
    } else if (line.startsWith("FNH:")) {
      functionsHit += Number(line.slice(4));
    } else if (line.startsWith("BRF:")) {
      branchesFound += Number(line.slice(4));
    } else if (line.startsWith("BRH:")) {
      branchesHit += Number(line.slice(4));
    }
  }

  const pct = (hit, total) => (total === 0 ? 0 : (hit / total) * 100);

  return {
    lines: pct(linesHit, linesFound),
    functions: pct(functionsHit, functionsFound),
    branches: pct(branchesHit, branchesFound),
  };
};

const loadYamlThresholds = async (filePath) => {
  try {
    const text = await fs.readFile(filePath, "utf8");
    const readValue = (section, key) => {
      const pattern = new RegExp(
        `${section}:[\\s\\S]*?threshold:[\\s\\S]*?${key}:\\s*(\\d+)`,
        "m",
      );
      const match = text.match(pattern);
      return match ? Number(match[1]) : null;
    };

    return {
      rust: {
        line: readValue("rust", "line"),
        function: readValue("rust", "function"),
        branch: readValue("rust", "branch"),
      },
      typescript: {
        line: readValue("typescript", "line"),
        function: readValue("typescript", "function"),
        branch: readValue("typescript", "branch"),
      },
      e2e: {
        line: readValue("playwright", "line"),
        function: readValue("playwright", "function"),
      },
    };
  } catch {
    return null;
  }
};

const applyOverrides = (thresholds, overrides) => {
  const next = structuredClone(thresholds);
  for (const [section, values] of Object.entries(overrides)) {
    for (const [key, value] of Object.entries(values)) {
      if (typeof value === "number" && !Number.isNaN(value)) {
        next[section][key] = value;
      }
    }
  }
  return next;
};

const defaults = {
  rust: { line: 90, function: 85, branch: 80 },
  typescript: { line: 85, function: 80, branch: 75 },
  e2e: { line: 70, function: 65 },
};

const yamlThresholds = await loadYamlThresholds(
  process.env.TEST_CONFIG_FILE || "test-config.yaml",
);
const thresholds = applyOverrides(defaults, yamlThresholds ?? {});

const overrides = {
  rust: {
    line: Number(process.env.RUST_COVERAGE_LINE),
    function: Number(process.env.RUST_COVERAGE_FUNCTION),
    branch: Number(process.env.RUST_COVERAGE_BRANCH),
  },
  typescript: {
    line: Number(process.env.TS_COVERAGE_LINE),
    function: Number(process.env.TS_COVERAGE_FUNCTION),
    branch: Number(process.env.TS_COVERAGE_BRANCH),
  },
  e2e: {
    line: Number(process.env.E2E_COVERAGE_LINE),
    function: Number(process.env.E2E_COVERAGE_FUNCTION),
  },
};

const finalThresholds = applyOverrides(thresholds, overrides);

const failures = [];

const checkMetric = (label, actual, required) => {
  if (actual < required) {
    failures.push(`${label} ${actual.toFixed(2)}% < ${required}%`);
  }
};

const rustLcov = path.join(projectRoot, "tests/coverage/rust/lcov.info");
try {
  const rustCoverage = await parseLcov(rustLcov);
  checkMetric("Rust line", rustCoverage.lines, finalThresholds.rust.line);
  checkMetric("Rust function", rustCoverage.functions, finalThresholds.rust.function);
  checkMetric("Rust branch", rustCoverage.branches, finalThresholds.rust.branch);
} catch {
  failures.push(`Rust coverage missing at ${rustLcov}`);
}

const tsLcov = path.join(projectRoot, "tests/coverage/typescript/lcov.info");
try {
  const tsCoverage = await parseLcov(tsLcov);
  checkMetric(
    "TypeScript line",
    tsCoverage.lines,
    finalThresholds.typescript.line,
  );
  checkMetric(
    "TypeScript function",
    tsCoverage.functions,
    finalThresholds.typescript.function,
  );
  checkMetric(
    "TypeScript branch",
    tsCoverage.branches,
    finalThresholds.typescript.branch,
  );
} catch {
  failures.push(`TypeScript coverage missing at ${tsLcov}`);
}

const e2eSummaryPath = path.join(
  projectRoot,
  "tests/coverage/e2e/coverage-report/coverage-summary.json",
);
try {
  const raw = await fs.readFile(e2eSummaryPath, "utf8");
  const summary = JSON.parse(raw);
  const linePct = summary?.coverage?.lines?.pct ?? 0;
  const functionPct = summary?.coverage?.functions?.pct ?? 0;
  checkMetric("E2E line", linePct, finalThresholds.e2e.line);
  checkMetric("E2E function", functionPct, finalThresholds.e2e.function);
} catch {
  failures.push(`E2E coverage summary missing at ${e2eSummaryPath}`);
}

if (failures.length > 0) {
  console.error("Coverage thresholds not met:");
  for (const failure of failures) {
    console.error(`- ${failure}`);
  }
  process.exit(1);
}

console.log("Coverage thresholds met.");
