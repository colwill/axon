# AXON — AI eXchange Optimised Notation

A pure Rust rule-based NLP engine that translates natural language into AXON notation, compiled to WebAssembly, runs entirely in the browser. Handles both general knowledge and programming tasks in one unified notation.

## Stack

- **Translation engine**: Pure Rust rule-based NLP (`src/translator.rs` + `src/code_translator.rs`)
- **WASM bindings**: `wasm-bindgen`
- **Frontend**: Vanilla HTML + JS (`web/index.html`)

## Setup

### Prerequisites

- Rust (install via https://rustup.rs)
- `wasm-pack` (`cargo install wasm-pack`)

### Build

```bash
make build
```

Compiles the Rust library to WebAssembly and outputs the package to `web/pkg/`.

### Develop

```bash
make serve
```

Builds the WASM artifact then serves the `web/` directory at **http://localhost:3000**.

### Test

```bash
cargo test
```

Runs 700+ test cases including general NLP and programming query translation.

## Project Structure

```
axon/
├── Cargo.toml              # Library crate: cdylib + wasm-bindgen
├── Makefile                # build / serve targets
├── src/
│   ├── lib.rs              # wasm-bindgen exports
│   ├── translator.rs       # General NLP → AXON translation engine
│   └── code_translator.rs  # Programming query & structural operation translator
├── tests/
│   └── code_questions.rs   # 700 programming question test cases
└── web/
    ├── index.html          # Browser frontend
    └── pkg/                # wasm-pack build output (generated)
```

## How the Translation Engine Works

The engine runs a unified pipeline:

**Stage 0 — Programming detection** (`code_translator.rs`):
- Commands: `"fix the bug in the auth service"` → `>fix bug:auth-service`
- Queries: `"what is the best way to cache"` → `?best cache`
- Structural: `"add a field email to User"` → `@user+.email`
- If no programming pattern matches, falls through to the general pipeline.

**Stages 1–5 — General NLP** (`translator.rs`):
1. **Confidence extraction** — hedge words (`probably` → `*`, `certainly` → `!!`)
2. **Temporal extraction** — time phrases (`today` → `^now`, `in 3 days` → `^T+3d`)
3. **Negation detection** — `not`, `no evidence`, `without` → `¬` / `∅`
4. **Structural parsing** — causal (`→`), logical (`∴` `∵` `∧` `∨`) patterns
5. **Token tagging** — classifies tokens: `@entity` `#concept` `~process` `$value` `∅null`

## AXON Quick Reference

```
Type sigils:    @entity  #concept  ~process  >command  ?query  $scalar  ^temporal  .member  ∅null
Operators:      →  ←  :  =  +  -  <  ↔  ≡  ∴  ∵  ¬  ∧  ∨  ⊕  ∀  ∃  Δ
Confidence:     !!  !  ~  *  **
Commands:       >doc >impl >fix >test >rev >ref >opt >plan >dep >add >rm >up >mv >cfg >mig
                >db >api >ci >sec >err >log >bench >lint >merge
Queries:        ?how ?why ?best ?what ?diff ?when ?where ?can ?cmp ?alt ?err ?perf
Structural:     @Type+.field  @Type-.field  @Type.x=$v  @Type.x:T  @Type:impl(@Trait)  @A<@B
```

Examples:
- `@CO2-emission → #climate-change!! → Δ$temp↑`
- `>doc login-component:auth-service`
- `?best implement-caching`
- `@user+.email`
- `@user:impl(@serializable)`
