// stats.js - Client-side logic for comP Statistics Dashboard
//
// Responsibilities:
// - Receive messages from VSCode extension (stats updates)
// - Update UI elements in real-time
// - Handle button clicks (refresh, force re-index)
// - Format and display language breakdown
// - Update status badges (Ready, Indexing, Error)

/**
 * Message channel to VSCode extension
 */
const vscode = acquireVsCodeApi();

/**
 * UI element references
 */
const elements = {
    totalFiles: document.getElementById("totalFiles"),
    totalNodes: document.getElementById("totalNodes"),
    totalEdges: document.getElementById("totalEdges"),
    indexSize: document.getElementById("indexSize"),
    languagesList: document.getElementById("languagesList"),
    lastIndexed: document.getElementById("lastIndexed"),
    indexStatus: document.getElementById("indexStatus"),
    refreshBtn: document.getElementById("refreshBtn"),
    forceIndexBtn: document.getElementById("forceIndexBtn"),
};

/**
 * Initialize event listeners
 */
function initializeEventListeners() {
    elements.refreshBtn?.addEventListener("click", () => {
        vscode.postMessage({ command: "refresh" });
    });

    elements.forceIndexBtn?.addEventListener("click", () => {
        vscode.postMessage({ command: "forceIndex" });
    });
}

/**
 * Handle messages from VSCode extension
 *
 * Message types:
 * - statsUpdate: Index statistics have changed
 * - indexStarted: Re-indexing has started
 * - indexProgress: Re-indexing progress update
 * - indexCompleted: Re-indexing finished
 * - error: An error occurred
 */
window.addEventListener("message", (event) => {
    const message = event.data;

    switch (message.type) {
        case "statsUpdate":
            updateStats(message.stats);
            break;
        case "indexStarted":
            setIndexing(true);
            break;
        case "indexProgress":
            updateProgress(message.current, message.total);
            break;
        case "indexCompleted":
            setIndexing(false);
            updateStats(message.stats);
            break;
        case "error":
            handleError(message.error);
            break;
    }
});

/**
 * Update statistics display
 *
 * @param {Object} stats - Statistics object
 * @param {number} stats.totalFiles - Number of indexed files
 * @param {number} stats.totalNodes - Number of symbols found
 * @param {number} stats.totalEdges - Number of dependencies
 * @param {string} stats.indexSize - Database size (e.g., "2.5 MB")
 * @param {Object} stats.languages - Language breakdown
 * @param {string} stats.lastIndexed - Last indexed timestamp
 */
function updateStats(stats) {
    // Update stat cards
    if (elements.totalFiles) elements.totalFiles.textContent = formatNumber(stats.totalFiles);
    if (elements.totalNodes) elements.totalNodes.textContent = formatNumber(stats.totalNodes);
    if (elements.totalEdges) elements.totalEdges.textContent = formatNumber(stats.totalEdges);
    if (elements.indexSize) elements.indexSize.textContent = stats.indexSize || "--";

    // Update languages
    if (elements.languagesList) {
        elements.languagesList.innerHTML = renderLanguages(stats.languages);
    }

    // Update status
    if (elements.lastIndexed) {
        elements.lastIndexed.textContent = formatTimestamp(stats.lastIndexed);
    }

    // Update status badge
    if (elements.indexStatus) {
        elements.indexStatus.textContent = "Ready";
        elements.indexStatus.className = "status-badge ready";
    }

    // Enable buttons
    elements.refreshBtn.disabled = false;
    elements.forceIndexBtn.disabled = false;
}

/**
 * Render language breakdown as badges
 *
 * @param {Object} languages - Language counts
 * @returns {string} HTML for language badges
 */
function renderLanguages(languages) {
    if (!languages || Object.keys(languages).length === 0) {
        return '<p class="loading">No files indexed yet</p>';
    }

    return Object.entries(languages)
        .sort(([, a], [, b]) => b - a) // Sort by count descending
        .map(
            ([lang, count]) =>
                `<div class="language-badge">
            <span class="name">${capitalizeFirst(lang)}</span>
            <span class="count">${count}</span>
        </div>`
        )
        .join("");
}

/**
 * Set indexing status
 *
 * @param {boolean} isIndexing - Whether currently indexing
 */
function setIndexing(isIndexing) {
    if (elements.indexStatus) {
        if (isIndexing) {
            elements.indexStatus.textContent = "Indexing";
            elements.indexStatus.className = "status-badge indexing";
        } else {
            elements.indexStatus.textContent = "Ready";
            elements.indexStatus.className = "status-badge ready";
        }
    }

    // Disable buttons during indexing
    elements.refreshBtn.disabled = isIndexing;
    elements.forceIndexBtn.disabled = isIndexing;
}

/**
 * Update indexing progress
 *
 * @param {number} current - Current file number
 * @param {number} total - Total files to index
 */
function updateProgress(current, total) {
    setIndexing(true);
    if (elements.indexStatus) {
        const percent = Math.round((current / total) * 100);
        elements.indexStatus.textContent = `Indexing ${percent}%`;
    }
}

/**
 * Handle errors from extension
 *
 * @param {string} error - Error message
 */
function handleError(error) {
    console.error("[comP Dashboard]", error);

    if (elements.indexStatus) {
        elements.indexStatus.textContent = "Error";
        elements.indexStatus.className = "status-badge error";
    }

    // Show error alert (only visible in dev console for now)
    alert(`comP Error: ${error}`);
}

/**
 * Format a number with thousands separator
 */
function formatNumber(num) {
    return new Intl.NumberFormat("en-US").format(num);
}

/**
 * Format a timestamp
 */
function formatTimestamp(timestamp) {
    if (!timestamp || timestamp === "0") {
        return "Never";
    }

    try {
        const date = new Date(timestamp);
        return date.toLocaleString();
    } catch {
        return timestamp;
    }
}

/**
 * Capitalize first letter
 */
function capitalizeFirst(str) {
    return str.charAt(0).toUpperCase() + str.slice(1);
}

/**
 * Initialize on page load
 */
document.addEventListener("DOMContentLoaded", () => {
    initializeEventListeners();

    // Request initial stats
    vscode.postMessage({ command: "refresh" });
});
