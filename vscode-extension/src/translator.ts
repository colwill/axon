import { join } from "path";
import { spawn, spawnSync, ChildProcess } from "child_process";
import { createInterface, Interface } from "readline";
import { existsSync } from "fs";

export interface TranslationResult {
  axon: string;
  annotation: string;
  savings: number;
  free(): void;
}

export interface CompressResult {
  encoded: string;
  original_bytes: number;
  compressed_bytes: number;
  ratio: number;
}

export interface Translator {
  translate(input: string): Promise<TranslationResult>;
  decode(axon: string): Promise<string>;
  tokens(text: string): Promise<number>;
  compress(text: string): Promise<CompressResult>;
  dispose(): void;
}

// ─── REPL-based translator (preferred) ──────────────────────────────────────

/**
 * Spawns `axon-repl --json` as a long-running subprocess and communicates
 * via JSON lines on stdin/stdout. Provides encode, decode, tokens, and compress.
 */
class ReplTranslator implements Translator {
  private proc: ChildProcess;
  private rl: Interface;
  private pending: Array<(line: string) => void> = [];
  private dead = false;

  constructor(binaryPath: string) {
    this.proc = spawn(binaryPath, ["--json"], {
      stdio: ["pipe", "pipe", "ignore"],
    });

    this.rl = createInterface({ input: this.proc.stdout! });

    this.rl.on("line", (line: string) => {
      const resolve = this.pending.shift();
      if (resolve) resolve(line);
    });

    this.proc.on("close", () => {
      this.dead = true;
      // Reject any remaining pending requests
      for (const resolve of this.pending) {
        resolve('{"ok":false,"error":"REPL process exited"}');
      }
      this.pending = [];
    });

    this.proc.on("error", () => {
      this.dead = true;
    });
  }

  private send(request: object): Promise<string> {
    return new Promise((resolve) => {
      if (this.dead || !this.proc.stdin?.writable) {
        resolve('{"ok":false,"error":"REPL process not running"}');
        return;
      }
      this.pending.push(resolve);
      this.proc.stdin!.write(JSON.stringify(request) + "\n");
    });
  }

  async translate(input: string): Promise<TranslationResult> {
    const line = await this.send({ action: "encode", text: input });
    try {
      const resp = JSON.parse(line);
      if (resp.ok) {
        return {
          axon: resp.axon,
          annotation: resp.annotation,
          savings: resp.savings_pct,
          free: () => {},
        };
      }
    } catch {}
    return { axon: input, annotation: "repl-error", savings: 0, free: () => {} };
  }

  async decode(axon: string): Promise<string> {
    const line = await this.send({ action: "decode", axon });
    try {
      const resp = JSON.parse(line);
      if (resp.ok) return resp.text;
    } catch {}
    return axon;
  }

  async tokens(text: string): Promise<number> {
    const line = await this.send({ action: "tokens", text });
    try {
      const resp = JSON.parse(line);
      if (resp.ok) return resp.tokens;
    } catch {}
    return text.split(/\s+/).length;
  }

  async compress(text: string): Promise<CompressResult> {
    const line = await this.send({ action: "compress", text });
    try {
      const resp = JSON.parse(line);
      if (resp.ok) return resp;
    } catch {}
    return { encoded: text, original_bytes: text.length, compressed_bytes: text.length, ratio: 0 };
  }

  dispose() {
    this.proc.kill();
    this.rl.close();
  }
}

// ─── WASM-based translator (fallback) ───────────────────────────────────────

/**
 * Falls back to the WASM module if the REPL binary is not available.
 * Only supports translate (encode) — decode/tokens/compress return defaults.
 */
class WasmTranslator implements Translator {
  private inner: any;

  constructor(extensionPath: string) {
    const wasmGlue = require(join(extensionPath, "wasm", "axon.js"));
    this.inner = new wasmGlue.AxonTranslator();
  }

  async translate(input: string): Promise<TranslationResult> {
    const result = this.inner.translate(input);
    const out = {
      axon: result.axon as string,
      annotation: result.annotation as string,
      savings: result.savings as number,
      free: () => result.free(),
    };
    return out;
  }

  async decode(_axon: string): Promise<string> {
    return _axon; // WASM doesn't support decode
  }

  async tokens(text: string): Promise<number> {
    return text.split(/\s+/).filter(Boolean).length;
  }

  async compress(text: string): Promise<CompressResult> {
    return { encoded: text, original_bytes: text.length, compressed_bytes: text.length, ratio: 0 };
  }

  dispose() {}
}

// ─── Passthrough translator (last resort) ───────────────────────────────────

class PassthroughTranslator implements Translator {
  async translate(input: string): Promise<TranslationResult> {
    return { axon: input, annotation: "no-translator", savings: 0, free: () => {} };
  }
  async decode(axon: string): Promise<string> { return axon; }
  async tokens(text: string): Promise<number> { return text.split(/\s+/).filter(Boolean).length; }
  async compress(text: string): Promise<CompressResult> {
    return { encoded: text, original_bytes: text.length, compressed_bytes: text.length, ratio: 0 };
  }
  dispose() {}
}

// ─── Factory ────────────────────────────────────────────────────────────────

let translatorInstance: Translator | null = null;

/**
 * Check whether a binary name on PATH is executable.
 */
function isOnPath(name: string): boolean {
  try {
    const result = spawnSync(
      process.platform === "win32" ? "where" : "which",
      [name],
      { timeout: 2000, stdio: "ignore" }
    );
    return result.status === 0;
  } catch {
    return false;
  }
}

/**
 * Create a translator instance. Tries REPL binary first, then WASM, then passthrough.
 *
 * REPL binary search order:
 * 1. Workspace-relative `target/release/axon-repl`
 * 2. Workspace-relative `target/debug/axon-repl`
 * 3. Extension dev path `../target/release/axon-repl`
 * 4. Extension dev path `../target/debug/axon-repl`
 * 5. `axon-repl` on PATH
 */
export function getTranslator(extensionPath: string, workspaceRoot?: string): Translator {
  if (translatorInstance) return translatorInstance;

  // Extension dev root (extension is in vscode-extension/ subdir during development)
  const extensionDevRoot = join(extensionPath, "..");

  // Build candidate list: workspace paths first, then extension dev paths
  const candidatePaths: string[] = [];
  if (workspaceRoot) {
    candidatePaths.push(join(workspaceRoot, "target", "release", "axon-repl"));
    candidatePaths.push(join(workspaceRoot, "target", "debug", "axon-repl"));
  }
  candidatePaths.push(join(extensionDevRoot, "target", "release", "axon-repl"));
  candidatePaths.push(join(extensionDevRoot, "target", "debug", "axon-repl"));

  // Deduplicate (workspace root and extension dev root may be the same)
  const searchPaths = [...new Set(candidatePaths)];

  // Try absolute paths — only if the file actually exists
  for (const binPath of searchPaths) {
    if (!existsSync(binPath)) {
      console.log(`AXON: Binary not found at ${binPath}`);
      continue;
    }
    try {
      const t = new ReplTranslator(binPath);
      translatorInstance = t;
      console.log(`AXON: Using REPL translator at ${binPath}`);
      return t;
    } catch {
      continue;
    }
  }

  // Try PATH fallback — only if axon-repl is actually on PATH
  if (isOnPath("axon-repl")) {
    try {
      const t = new ReplTranslator("axon-repl");
      translatorInstance = t;
      console.log("AXON: Using REPL translator from PATH");
      return t;
    } catch {
      // fall through
    }
  }

  // Try WASM fallback
  try {
    translatorInstance = new WasmTranslator(extensionPath);
    console.log("AXON: Using WASM translator (REPL binary not found)");
    return translatorInstance;
  } catch (err) {
    console.error("AXON: WASM load failed:", err);
  }

  // Last resort
  console.warn("AXON: No translator available, using passthrough");
  translatorInstance = new PassthroughTranslator();
  return translatorInstance;
}
