"use strict";

const elements = {
  serverUrl: document.querySelector("#server-url"),
  connectButton: document.querySelector("#connect-button"),
  refreshButton: document.querySelector("#refresh-button"),
  connectionDot: document.querySelector("#connection-dot"),
  connectionStatus: document.querySelector("#connection-status"),
  objectiveCount: document.querySelector("#objective-count"),
  queuedCount: document.querySelector("#queued-count"),
  activeCount: document.querySelector("#active-count"),
  eventCount: document.querySelector("#event-count"),
  objectiveList: document.querySelector("#objective-list"),
  objectiveForm: document.querySelector("#objective-form"),
  repository: document.querySelector("#repository"),
  description: document.querySelector("#description"),
  branch: document.querySelector("#branch"),
  submitButton: document.querySelector("#submit-button"),
  formStatus: document.querySelector("#form-status"),
  eventList: document.querySelector("#event-list"),
  clearEventsButton: document.querySelector("#clear-events-button"),
  objectiveTemplate: document.querySelector("#objective-template"),
};

const state = {
  baseUrl: "",
  objectives: [],
  eventSource: null,
  eventCount: 0,
};

function normalizeServer(value) {
  const raw = value.trim();
  let parsed;
  try {
    parsed = new URL(raw);
  } catch {
    throw new Error("Enter a valid http:// or https:// server URL.");
  }
  if (parsed.protocol !== "http:" && parsed.protocol !== "https:") {
    throw new Error("Server URL must use http:// or https://.");
  }
  if (parsed.username || parsed.password) {
    throw new Error("Do not embed credentials in the server URL.");
  }
  parsed.hash = "";
  parsed.search = "";
  parsed.pathname = parsed.pathname.replace(/\/+$/, "");
  return parsed.toString().replace(/\/$/, "");
}

function endpoint(path) {
  if (!state.baseUrl) {
    throw new Error("Connect to a server first.");
  }
  return `${state.baseUrl}${path}`;
}

function setConnection(kind, message) {
  elements.connectionDot.classList.remove("connected", "error");
  if (kind) {
    elements.connectionDot.classList.add(kind);
  }
  elements.connectionStatus.textContent = message;
}

function setBusy(button, busy, label) {
  button.disabled = busy;
  if (label) {
    button.dataset.idleLabel ||= button.textContent;
    button.textContent = busy ? label : button.dataset.idleLabel;
  }
}

async function requestJson(path, options = {}) {
  const response = await fetch(endpoint(path), {
    ...options,
    headers: {
      accept: "application/json",
      ...(options.body ? { "content-type": "application/json" } : {}),
      ...(options.headers || {}),
    },
  });

  let payload;
  try {
    payload = await response.json();
  } catch {
    throw new Error(`Server returned HTTP ${response.status} with invalid JSON.`);
  }

  if (!response.ok) {
    const detail = payload && typeof payload.error === "string" ? `: ${payload.error}` : "";
    throw new Error(`HTTP ${response.status}${detail}`);
  }
  return payload;
}

function updateMetrics() {
  const queued = state.objectives.filter((item) => item.status === "queued").length;
  const active = state.objectives.filter((item) => ["running", "active"].includes(item.status)).length;
  elements.objectiveCount.textContent = String(state.objectives.length);
  elements.queuedCount.textContent = String(queued);
  elements.activeCount.textContent = String(active);
  elements.eventCount.textContent = String(state.eventCount);
}

function renderObjectives() {
  elements.objectiveList.replaceChildren();
  if (state.objectives.length === 0) {
    const empty = document.createElement("p");
    empty.className = "empty-state";
    empty.textContent = "No objectives are currently queued.";
    elements.objectiveList.append(empty);
    updateMetrics();
    return;
  }

  for (const objective of state.objectives) {
    const fragment = elements.objectiveTemplate.content.cloneNode(true);
    const status = fragment.querySelector(".objective-status");
    status.textContent = objective.status || "unknown";
    status.dataset.status = objective.status || "unknown";
    fragment.querySelector(".objective-id").textContent = objective.id || "";
    fragment.querySelector(".objective-description").textContent = objective.description || "Untitled objective";
    fragment.querySelector(".objective-repository").textContent = objective.repository || "—";
    fragment.querySelector(".objective-branch").textContent = objective.branch || "—";
    elements.objectiveList.append(fragment);
  }
  updateMetrics();
}

async function loadObjectives() {
  setBusy(elements.refreshButton, true, "Loading…");
  try {
    const payload = await requestJson("/api/v1/objectives");
    if (!Array.isArray(payload)) {
      throw new Error("Objective response is not an array.");
    }
    state.objectives = payload;
    renderObjectives();
    setConnection("connected", `Connected to ${state.baseUrl}`);
  } finally {
    setBusy(elements.refreshButton, false, "Loading…");
  }
}

function addEvent(raw) {
  state.eventCount += 1;
  updateMetrics();

  let display = raw;
  try {
    display = JSON.stringify(JSON.parse(raw));
  } catch {
    // Preserve non-JSON SSE payloads as received.
  }

  const item = document.createElement("li");
  const time = document.createElement("span");
  time.className = "event-time";
  time.textContent = new Date().toLocaleTimeString([], { hour12: false });
  const data = document.createElement("span");
  data.className = "event-data";
  data.textContent = display;
  item.append(time, data);
  elements.eventList.prepend(item);

  while (elements.eventList.children.length > 100) {
    elements.eventList.lastElementChild.remove();
  }
}

function connectEvents() {
  if (state.eventSource) {
    state.eventSource.close();
  }
  const source = new EventSource(endpoint("/events"));
  state.eventSource = source;
  source.onmessage = (event) => {
    addEvent(event.data);
    void loadObjectives().catch(() => {});
  };
  source.onopen = () => setConnection("connected", `Connected to ${state.baseUrl}`);
  source.onerror = () => setConnection("error", "Event stream disconnected; browser retry is active.");
}

async function connect() {
  setBusy(elements.connectButton, true, "Connecting…");
  try {
    state.baseUrl = normalizeServer(elements.serverUrl.value);
    localStorage.setItem("autodev.commandCenter.server", state.baseUrl);
    await loadObjectives();
    connectEvents();
  } catch (error) {
    setConnection("error", error instanceof Error ? error.message : String(error));
  } finally {
    setBusy(elements.connectButton, false, "Connecting…");
  }
}

async function submitObjective(event) {
  event.preventDefault();
  elements.formStatus.classList.remove("error");
  elements.formStatus.textContent = "";

  const repository = elements.repository.value.trim();
  const description = elements.description.value.trim();
  const branch = elements.branch.value.trim();
  if (!repository || !description) {
    elements.formStatus.classList.add("error");
    elements.formStatus.textContent = "Repository and description are required.";
    return;
  }

  setBusy(elements.submitButton, true, "Queueing…");
  try {
    const payload = { repository, description };
    if (branch) {
      payload.branch = branch;
    }
    const created = await requestJson("/api/v1/objectives", {
      method: "POST",
      body: JSON.stringify(payload),
    });
    elements.formStatus.textContent = `Queued ${created.id || "objective"}.`;
    elements.description.value = "";
    elements.branch.value = "";
    await loadObjectives();
  } catch (error) {
    elements.formStatus.classList.add("error");
    elements.formStatus.textContent = error instanceof Error ? error.message : String(error);
  } finally {
    setBusy(elements.submitButton, false, "Queueing…");
  }
}

function restoreServer() {
  const stored = localStorage.getItem("autodev.commandCenter.server");
  if (stored) {
    elements.serverUrl.value = stored;
  }
}

elements.connectButton.addEventListener("click", () => void connect());
elements.refreshButton.addEventListener("click", () => {
  void loadObjectives().catch((error) => {
    setConnection("error", error instanceof Error ? error.message : String(error));
  });
});
elements.objectiveForm.addEventListener("submit", (event) => void submitObjective(event));
elements.clearEventsButton.addEventListener("click", () => {
  elements.eventList.replaceChildren();
  state.eventCount = 0;
  updateMetrics();
});
window.addEventListener("beforeunload", () => state.eventSource?.close());

restoreServer();
updateMetrics();
