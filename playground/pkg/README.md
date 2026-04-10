# AXON — AI eXchange Optimised Notation

[AXON Playground and docs](https://colwill.github.io/axon/playground)

A pure Rust rule-based NLP engine that translates natural language into AXON notation, compiled to WebAssembly, runs entirely in the browser. Handles both general knowledge and programming tasks in one unified notation.

**Full specification:** [AXON_SPEC.md](AXON_SPEC.md)

VSCode package: https://marketplace.visualstudio.com/items?itemName=colwill.axon-notation


Launch VS Code Quick Open (Ctrl+P), paste the following command, and press enter.

Install command in vscode: `ext install colwill.axon-notation`

## Stack

- **Translation engine**: Pure Rust rule-based NLP (`lib/translator.rs` + `lib/code_translator.rs`)
- **WASM bindings**: `wasm-bindgen`
- **Frontend**: Vanilla HTML + JS (`playground/index.html`)

## Setup

### Prerequisites

- Rust (install via https://rustup.rs)
- `wasm-pack` (`cargo install wasm-pack`)

### Build

```bash
make build
```

Compiles the Rust library to WebAssembly and outputs the package to `playground/pkg/`.

### Develop

```bash
make serve
```

Builds the WASM artifact then serves the `playground/` directory at **http://localhost:3001**.

### Test

```bash
cargo test
```

Runs 700+ test cases including general NLP, programming query translation, and caveman benchmark comparisons.

## Project Structure

```
axon/
├── AXON_SPEC.md            # Single source of truth for the AXON notation spec
├── Cargo.toml              # Workspace root + library crate
├── Makefile                # build / serve targets
├── lib/
│   ├── lib.rs              # wasm-bindgen exports + BPE token estimation
│   ├── translator.rs       # General NLP -> AXON engine + abbreviation tiers
│   ├── code_translator.rs  # Programming query & structural operation translator
│   ├── huffman.rs          # Huffman compression
│   └── tests/
│       ├── code_questions.rs      # 700 programming question test cases
│       ├── caveman_comparison.rs  # Comparison tests vs caveman benchmark
│       └── fixtures/
│           └── caveman_results.json  # Caveman benchmark dataset
├── skills/
│   └── axon/
│       └── SKILL.md        # Claude Code skill definition
├── repl/                   # Interactive CLI translator
├── vscode-extension/       # VS Code extension
└── playground/
    ├── index.html          # Browser frontend (AXON Playground)
    └── pkg/                # wasm-pack build output (generated)
```

## How the Translation Engine Works

The engine runs a unified pipeline:

**Stage 0 — Programming detection** (`code_translator.rs`):
- Commands: `"fix the bug in the auth service"` -> `>fix bug:auth-service`
- Queries: `"what is the best way to implement caching"` -> `?best impl-caching`
- Structural: `"add a field email to User"` -> `@user+.email`
- If no programming pattern matches, falls through to the general pipeline.

**Stages 1-5 — General NLP** (`translator.rs`):
1. **Confidence extraction** — hedge words (`probably` -> `*`, `certainly` -> `!!`)
2. **Temporal extraction** — time phrases (`today` -> `^now`, `in 3 days` -> `^T+3d`)
3. **Negation detection** — `not`, `no evidence`, `without` -> `!` prefix
4. **Structural parsing** — causal (`->`), logical (`:.` `bc` `&&` `||`) patterns
5. **Token tagging** — classifies known tokens only: `@entity` `#concept` `~process` `$value`
6. **Abbreviation** — common terms shortened using the active abbreviation tier
7. **Compound merging** — consecutive bare tokens hyphenated: `new obj ref` -> `new-obj-ref`

## v1.1 Abbreviation Tiers

AXON v1.1 introduces four abbreviation levels, each inclusive of the previous:

| Level | Name | Terms | Description |
|-------|------|-------|-------------|
| L0 | Default | 85+ | Standard developer terms (object->obj, function->fn, database->db) |
| L1 | Extended | +90 | Networking, OS, data, DevOps (protocol->proto, process->proc, kubernetes->k8s) |
| L2 | Aggressive | +50 | Common English (between->btw, difference->diff, strategy->strat) |
| L3 | Maximum | +45 | Single-char extremes (function->f, variable->v, string->s) |

## Token Reduction Strategies

v1.1 implements six strategies to achieve genuine BPE token reduction:

### 1. ASCII Operators
All operators use ASCII characters instead of Unicode, avoiding multi-token BPE splits:
- `->` instead of `→`, `<-` instead of `←`
- `&&` instead of `∧`, `||` instead of `∨`
- `:.` instead of `∴`, `bc` instead of `∵`
- `A.` instead of `∀`, `E.` instead of `∃`

### 2. Tiered Abbreviation Dictionary
Four levels of abbreviation from conservative (L0) to extreme (L3), allowing users to choose the right compression/readability tradeoff.

### 3. Conditional Sigils
Sigil prefixes (`#`, `~`, `$`) are only applied to tokens found in the known entity/concept/verb databases. Unknown/fallback tokens are emitted bare (no prefix), saving 1 BPE token per word.

### 4. Stop-Word Stripping
Filler words, articles, copulas, and pleasantries are removed from both general NLP and programming queries, reducing noise without losing semantic content.

### 5. Compound Merging
Consecutive bare (unsigiled) tokens are merged into hyphenated compounds, reducing whitespace-separated token count: `new obj ref` -> `new-obj-ref` (3 tokens -> 1 token).

### 6. BPE-Aware Token Estimation
Token savings are calculated using BPE-aware estimation that counts non-alphanumeric characters as additional tokens, giving honest savings percentages.

## AXON Quick Reference

```
Type sigils:    @entity  #concept  ~process  >command  ?query  $scalar  ^temporal  .member
Operators:      ->  <-  :  =  +  -  <  &&  ||  :.  bc  A.  E.
Confidence:     !!  !  ~  *  **
Commands:       >doc >impl >fix >test >rev >ref >opt >plan >dep >add >rm >up >mv >cfg >mig
                >db >api >ci >sec >err >log >bench >lint >merge >explain
Queries:        ?how ?why ?best ?what ?diff ?when ?where ?can ?cmp ?alt ?err ?perf
Structural:     @Type+.field  @Type-.field  @Type.x=$v  @Type.x:T  @Type:impl(@Trait)  @A<@B
Abbrev tiers:   L0 (default)  L1 (extended)  L2 (aggressive)  L3 (maximum)
```

Examples:
- `@co2 #emission -> #climate-change!! -> #temperature ~rise`
- `>doc login-comp:auth-service`
- `>explain db-conn-pooling`
- `?best impl-caching`
- `@user+.email`
- `@user:impl(@serializable)`
- `A.new-obj-ref $rnd inl-obj-prop` (from: "New object ref each render. Inline object prop")
