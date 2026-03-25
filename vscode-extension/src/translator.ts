import { join } from "path";

interface TranslationResult {
  axon: string;
  annotation: string;
  savings: number;
  free(): void;
}

interface Translator {
  translate(input: string): TranslationResult;
}

let translatorInstance: Translator | null = null;

/**
 * Load the AXON WASM module and return a translator wrapper.
 * The --target nodejs wasm-pack output auto-loads the .wasm from __dirname.
 */
export function getTranslator(extensionPath: string): Translator {
  if (translatorInstance) {
    return translatorInstance;
  }

  try {
    // The nodejs target output self-initialises from __dirname/axon_bg.wasm
    const wasmGlue = require(join(extensionPath, "wasm", "axon.js"));
    const inner = new wasmGlue.AxonTranslator();

    translatorInstance = {
      translate(input: string): TranslationResult {
        const result = inner.translate(input);
        const out = {
          axon: result.axon as string,
          annotation: result.annotation as string,
          savings: result.savings as number,
          free: () => result.free(),
        };
        return out;
      },
    };

    return translatorInstance;
  } catch (err) {
    console.error("Failed to load AXON WASM module:", err);
    translatorInstance = {
      translate(input: string): TranslationResult {
        return {
          axon: input,
          annotation: "wasm-unavailable",
          savings: 0,
          free: () => {},
        };
      },
    };
    return translatorInstance;
  }
}
