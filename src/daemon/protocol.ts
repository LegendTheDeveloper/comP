// Protocol - Type definitions for daemon communication
//
// JSON-RPC 2.0 protocol for communication between:
// - VSCode extension (client) <-> Rust daemon (server)
// - Rust daemon (server) <-> AI agents (client)

// JSON-RPC request envelope
export interface JSONRPCRequest {
  jsonrpc: "2.0";
  id: string | number | null;
  method: string;
  params?: unknown;
}

// JSON-RPC response envelope
export interface JSONRPCResponse {
  jsonrpc: "2.0";
  id: string | number | null;
  result?: unknown;
  error?: JSONRPCError;
}

// JSON-RPC error
export interface JSONRPCError {
  code: number;
  message: string;
  data?: unknown;
}

// Daemon requests
export interface IndexRequest {
  workspace_root: string;
}

export interface IndexResponse {
  indexed_files: number;
  status: "completed" | "in_progress" | "error";
}

export interface StatsRequest {}

// Per-repo breakdown in the multi-repo index (workspace root + additional_paths).
export interface RepoStat {
  alias: string;
  root_path: string;
  files: number;
  nodes: number;
  is_root: boolean;
}

// Live indexing progress, polled via getStats so the sidebar panel can show
// which repo is currently being walked/parsed.
export interface IndexingInfo {
  is_indexing: boolean;
  current_repo: string | null;
}

export interface StatsResponse {
  total_files: number;
  total_nodes: number;
  total_edges: number;
  indexed_time_ms: number;
  repos?: RepoStat[];
  indexing?: IndexingInfo;
}

export interface AddRepoRequest {
  path: string;
}

export interface AddRepoResponse {
  status: "ok";
  alias: string;
  root_path: string;
}

export interface RemoveRepoRequest {
  alias: string;
}

export interface RemoveRepoResponse {
  status: "ok";
  alias: string;
  removed_files: number;
}
