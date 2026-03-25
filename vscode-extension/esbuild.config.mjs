import * as esbuild from "esbuild";
import { copyFileSync, mkdirSync, existsSync } from "fs";
import { join, dirname } from "path";
import { fileURLToPath } from "url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const watch = process.argv.includes("--watch");

// Copy WASM binary to dist/
const wasmSrc = join(__dirname, "wasm", "axon_bg.wasm");
const wasmDst = join(__dirname, "dist", "axon_bg.wasm");
mkdirSync(join(__dirname, "dist"), { recursive: true });
if (existsSync(wasmSrc)) {
  copyFileSync(wasmSrc, wasmDst);
}

const config = {
  entryPoints: ["src/extension.ts"],
  bundle: true,
  outfile: "dist/extension.js",
  external: ["vscode"],
  format: "cjs",
  platform: "node",
  target: "node18",
  sourcemap: true,
  tsconfig: "tsconfig.json",
};

if (watch) {
  const ctx = await esbuild.context(config);
  await ctx.watch();
  console.log("Watching for changes...");
} else {
  await esbuild.build(config);
  console.log("Build complete.");
}
