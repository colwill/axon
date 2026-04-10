/* tslint:disable */
/* eslint-disable */

export class AxonTranslator {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Compress text using Huffman encoding for smaller context usage.
     */
    compress(text: string): CompressionResult;
    constructor();
    translate(input: string): TranslationResult;
}

export class CompressionResult {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    readonly compressed_bytes: number;
    readonly encoded: string;
    readonly original_bytes: number;
    readonly ratio: number;
}

export class TranslationResult {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    readonly annotation: string;
    readonly axon: string;
    readonly savings: number;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_axontranslator_free: (a: number, b: number) => void;
    readonly __wbg_compressionresult_free: (a: number, b: number) => void;
    readonly __wbg_translationresult_free: (a: number, b: number) => void;
    readonly axontranslator_compress: (a: number, b: number, c: number) => number;
    readonly axontranslator_new: () => number;
    readonly axontranslator_translate: (a: number, b: number, c: number) => number;
    readonly compressionresult_compressed_bytes: (a: number) => number;
    readonly compressionresult_encoded: (a: number) => [number, number];
    readonly compressionresult_original_bytes: (a: number) => number;
    readonly compressionresult_ratio: (a: number) => number;
    readonly translationresult_annotation: (a: number) => [number, number];
    readonly translationresult_axon: (a: number) => [number, number];
    readonly translationresult_savings: (a: number) => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
