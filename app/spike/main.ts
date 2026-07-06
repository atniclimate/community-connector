import { createSpikeData } from "./data";
import {
  continueAutoRun,
  getCandidateFromUrl,
  isAutoRun,
  logResult,
  readStoredResults,
  runMeasurement,
  startAutoRun,
  type CandidateFactory,
  type CandidateName,
  type SpikeResult,
} from "./harness";
import { createForceGraphCandidate } from "./candidates/forcegraph";
import { createInstancedCandidate } from "./candidates/instanced";
import { createStockCandidate } from "./candidates/stock";

const factories: Record<CandidateName, CandidateFactory> = {
  stock: createStockCandidate,
  forcegraph: createForceGraphCandidate,
  instanced: createInstancedCandidate,
};

const candidateSelect = requireElement<HTMLSelectElement>("candidate");
const runAllButton = requireElement<HTMLButtonElement>("run-all");
const status = requireElement<HTMLElement>("status");
const container = requireElement<HTMLElement>("canvas-container");
const resultsBody = requireElement<HTMLTableSectionElement>("results-body");
const urlCandidate = getCandidateFromUrl();
const selectedCandidate = urlCandidate ?? "instanced";

candidateSelect.value = selectedCandidate;
renderResults(readStoredResults());

candidateSelect.addEventListener("change", () => {
  const candidate = parseCandidate(candidateSelect.value);
  window.location.href = `${window.location.pathname}?candidate=${candidate}`;
});

runAllButton.addEventListener("click", () => {
  status.textContent = "Starting auto run";
  startAutoRun();
});

void runSelected(selectedCandidate);

async function runSelected(candidate: CandidateName): Promise<void> {
  status.textContent = `Building ${candidate} dataset and renderer`;
  const data = createSpikeData();
  const handle = factories[candidate](container, data);

  try {
    status.textContent = `Measuring ${candidate}: 3s warmup, 20s run`;
    const result = await runMeasurement(candidate, handle);
    logResult(result);
    status.textContent = `${candidate} complete: ${result.avgFps} FPS avg, p95 ${result.p95FrameMs} ms`;

    if (isAutoRun()) {
      const navigating = continueAutoRun(result);
      if (!navigating) {
        renderResults(readStoredResults());
      }
      return;
    }

    renderResults([result]);
  } finally {
    handle.dispose();
  }
}

function renderResults(results: SpikeResult[]): void {
  resultsBody.replaceChildren(
    ...results.map((result) => {
      const row = document.createElement("tr");
      row.append(
        cell(result.candidate),
        cell(String(result.frames)),
        cell(result.avgFps.toFixed(2)),
        cell(result.p95FrameMs.toFixed(2)),
        cell(result.worstFrameMs.toFixed(2)),
        cell(result.drawCalls === null ? "n/a" : String(result.drawCalls)),
      );
      return row;
    }),
  );
}

function cell(text: string): HTMLTableCellElement {
  const td = document.createElement("td");
  td.textContent = text;
  return td;
}

function requireElement<T extends HTMLElement>(id: string): T {
  const element = document.getElementById(id);
  if (element === null) {
    throw new Error(`Missing #${id}`);
  }
  return element as T;
}

function parseCandidate(value: string): CandidateName {
  if (value === "stock" || value === "forcegraph" || value === "instanced") {
    return value;
  }
  throw new Error(`Unknown spike candidate: ${value}`);
}
