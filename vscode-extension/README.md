# AXON Notation — VS Code Extension

Translate natural language to **AXON notation** before sending prompts to AI — saving tokens on every message.

AXON (AI eXchange Optimised Notation) is a compact symbolic language that compresses natural language into dense, structured tokens. The translation runs locally via a Rust-compiled WebAssembly engine — no network requests, no API keys.

## Features

### Chat Participant (`@axon`)

Type `@axon` in VS Code's chat panel to route your prompts through AXON. Your input is translated to compact notation, sent to the language model, and the response is returned as normal — with a token savings percentage shown on each message.

### Sidebar Chat

A dedicated AXON chat panel in the activity bar. Open it with `Ctrl+Shift+A` (`Cmd+Shift+A` on macOS) or via the command palette.

### Commands

| Command | Description |
|---------|-------------|
| `AXON: Open Chat` | Focus the AXON sidebar panel |
| `AXON: Open Chat in Editor Tab` | Open the chat in a full editor tab |
| `AXON: Translate to AXON` | Translate text and choose where to send it |
| `AXON: Translate and Send to Claude Code` | Translate and send directly to Claude Code |
| `AXON: Translate and Send to Copilot Chat` | Translate and send to GitHub Copilot |
| `AXON: Translate and Copy to Clipboard` | Translate and copy the result |

### Keybinding

- `Ctrl+Shift+A` / `Cmd+Shift+A` — Open AXON Chat

## How It Works

1. You type a natural language prompt
2. The WASM engine translates it to AXON notation (e.g. `"create documentation for the auth service"` becomes `>doc auth-service`)
3. The compressed prompt is sent to your AI assistant
4. You get the same response with fewer tokens consumed

### Translation Examples

| Natural Language | AXON |
|-----------------|------|
| `what is the best way to implement caching` | `?best implement-caching` |
| `fix the bug in the auth service` | `>fix bug:auth-service` |
| `add a field email to User` | `@user+.email` |
| `CO2 emissions cause climate change which increases temperature` | `@CO2-emission → #climate-change!! → Δ$temp↑` |

## Settings

| Setting | Default | Description |
|---------|---------|-------------|
| `axon.defaultTarget` | `chat` | Default target for translated output: `chat`, `clipboard`, `claudeCode`, or `copilot` |

## Requirements

- VS Code 1.93.0 or later

## License

MIT
