"use strict";
var __create = Object.create;
var __defProp = Object.defineProperty;
var __getOwnPropDesc = Object.getOwnPropertyDescriptor;
var __getOwnPropNames = Object.getOwnPropertyNames;
var __getProtoOf = Object.getPrototypeOf;
var __hasOwnProp = Object.prototype.hasOwnProperty;
var __export = (target, all) => {
  for (var name in all)
    __defProp(target, name, { get: all[name], enumerable: true });
};
var __copyProps = (to, from, except, desc) => {
  if (from && typeof from === "object" || typeof from === "function") {
    for (let key of __getOwnPropNames(from))
      if (!__hasOwnProp.call(to, key) && key !== except)
        __defProp(to, key, { get: () => from[key], enumerable: !(desc = __getOwnPropDesc(from, key)) || desc.enumerable });
  }
  return to;
};
var __toESM = (mod, isNodeMode, target) => (target = mod != null ? __create(__getProtoOf(mod)) : {}, __copyProps(
  // If the importer is in node compatibility mode or this is not an ESM
  // file that has been converted to a CommonJS file using a Babel-
  // compatible transform (i.e. "__esModule" has not been set), then set
  // "default" to the CommonJS "module.exports" for node compatibility.
  isNodeMode || !mod || !mod.__esModule ? __defProp(target, "default", { value: mod, enumerable: true }) : target,
  mod
));
var __toCommonJS = (mod) => __copyProps(__defProp({}, "__esModule", { value: true }), mod);

// src/extension.ts
var extension_exports = {};
__export(extension_exports, {
  activate: () => activate,
  deactivate: () => deactivate
});
module.exports = __toCommonJS(extension_exports);
var vscode = __toESM(require("vscode"));
var path = __toESM(require("path"));
var fs = __toESM(require("fs"));
var import_child_process = require("child_process");

// src/translator.ts
var import_path = require("path");
var translatorInstance = null;
function getTranslator(extensionPath) {
  if (translatorInstance) {
    return translatorInstance;
  }
  try {
    const wasmGlue = require((0, import_path.join)(extensionPath, "wasm", "axon.js"));
    const inner = new wasmGlue.AxonTranslator();
    translatorInstance = {
      translate(input) {
        const result = inner.translate(input);
        const out = {
          axon: result.axon,
          annotation: result.annotation,
          savings: result.savings,
          free: () => result.free()
        };
        return out;
      }
    };
    return translatorInstance;
  } catch (err) {
    console.error("Failed to load AXON WASM module:", err);
    translatorInstance = {
      translate(input) {
        return {
          axon: input,
          annotation: "wasm-unavailable",
          savings: 0,
          free: () => {
          }
        };
      }
    };
    return translatorInstance;
  }
}

// src/extension.ts
var ChatHistory = class {
  entries = [];
  filePath;
  constructor(storagePath) {
    fs.mkdirSync(storagePath, { recursive: true });
    this.filePath = path.join(storagePath, "chat-history.json");
    this.load();
  }
  load() {
    try {
      if (fs.existsSync(this.filePath)) {
        this.entries = JSON.parse(fs.readFileSync(this.filePath, "utf-8"));
      }
    } catch {
      this.entries = [];
    }
  }
  save() {
    fs.writeFileSync(this.filePath, JSON.stringify(this.entries, null, 2));
  }
  add(entry) {
    const full = {
      id: `${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
      timestamp: Date.now(),
      ...entry
    };
    this.entries.push(full);
    this.save();
    return full;
  }
  updateResponse(id, response) {
    const entry = this.entries.find((e) => e.id === id);
    if (entry) {
      entry.response = response;
      this.save();
    }
  }
  search(query) {
    const q = query.toLowerCase();
    return this.entries.filter(
      (e) => e.userInput.toLowerCase().includes(q) || e.axon.toLowerCase().includes(q) || e.response.toLowerCase().includes(q)
    ).reverse().slice(0, 50);
  }
  recent(count = 20) {
    return this.entries.slice(-count).reverse();
  }
};
var AXON_SYSTEM_PROMPT = `You are fluent in AXON (AI eXchange Optimized Notation), a compact symbolic language designed for precise, token-efficient communication between humans and AI systems. You can encode natural language into AXON and decode AXON back into natural language.

## AXON Specification v1.0

### Type Sigils

Every meaningful token is prefixed with a sigil indicating its semantic type:

  @  Entity / Agent      \u2014 A named actor, system, or proper noun: @sun, @OpenAI, @user
  #  Concept / Abstract  \u2014 An idea, category, or domain: #gravity, #justice, #climate
  ~  Process / Action    \u2014 A verb, transformation, or operation: ~emit, ~learn, ~fail
  ?  Query / Unknown     \u2014 An open question or unresolved value: ?cause, ?result
  !  Assert              \u2014 A high-confidence factual claim: !true, !confirmed
  %  Quantifier          \u2014 A proportion, count, or frequency: %all, %few, %0.73
  ^  Temporal            \u2014 A time reference or duration: ^now, ^T-2d, ^T+1mo
  $  Scalar              \u2014 A measurable value or magnitude: $high, $3.14, $low
  \u2248  Approximate         \u2014 A fuzzy match or rough equivalence: \u2248#similar, \u2248$100
  \u2205  Null / Absent       \u2014 Absence, void, or negated entity: \u2205evidence, \u2205data

### Logical & Relational Operators

  \u2192   Causes / leads to         \u2190   Result of / caused by
  \u2194   Mutual / bidirectional    \u2261   Definitional equivalence
  \u2234   Therefore (conclusion)    \u2235   Because (premise/reason)
  \xAC   Not (negation)            \u2227   And (conjunction)
  \u2228   Or (disjunction)          \u2295   Xor (exclusive or)
  \u2283   Contains / superset       \u2200   For all (universal)
  \u2203   Exists (existential)      \u0394   Delta (change)
  \u2211   Sum / aggregate

### Epistemic Confidence Markers

  !!   Certain      \u2014 Verified fact, no doubt
  !    High         \u2014 Strong supporting evidence
  ~    Moderate     \u2014 Plausible, some uncertainty
  *    Low          \u2014 Weak evidence or guess
  **   Speculative  \u2014 Hypothetical or extrapolated
  ?    Unknown      \u2014 Insufficient data to assess

### Temporal Markers

  ^now         The current moment
  ^T-Nd        N days in the past (e.g., ^T-7d = one week ago)
  ^T+Nd        N days in the future (e.g., ^T+30d = next month)
  ^T+Nmo       N months in the future
  ^T-Ny        N years in the past
  ^\u2200t          All time \u2014 universally/always true
  ^span[A,B]   Time range from A to B

### Grammar Pattern

  [QUANTIFIER] [SUBJECT sigil+name] [OPERATOR] [OBJECT sigil+name] [CONFIDENCE] [TEMPORAL]

Parentheses group sub-expressions: ~fail(@server) \u2295 ~down(#network)
Multi-word tokens use hyphens: #climate-change, @milky-way

### Command Verbs (Programming)

  >doc   Document          >impl  Implement         >fix   Fix/debug
  >test  Write tests       >rev   Review            >ref   Refactor
  >opt   Optimise          >plan  Plan              >dep   Deploy
  >add   Add               >rm    Remove            >up    Update
  >mv    Move/rename       >cfg   Configure         >mig   Migrate
  >db    Database           >api   API               >ci    CI/CD
  >sec   Security          >err   Error handling    >log   Logging
  >bench Benchmark         >lint  Lint              >merge Merge

### Query Types (Programming)

  ?how   How to            ?why   Why does           ?best  Best approach
  ?what  What is           ?diff  Difference         ?when  When to
  ?where Where is          ?can   Can it             ?cmp   Compare
  ?alt   Alternatives      ?err   Error cause        ?perf  Performance

### Structural Operations (Programming)

  @Type+.field          Add field to type
  @Type-.field          Remove field from type
  @Type.x=$v            Set field value
  @Type.x:T             Set field type
  @Type:impl(@Trait)    Implement trait/interface
  @A<@B                 A extends/inherits B

### Encoding Rules (Natural Language \u2192 AXON)

1. Identify named entities, people, systems, organizations \u2192 prefix with @
2. Identify abstract concepts, ideas, categories, domains \u2192 prefix with #
3. Identify verbs, actions, processes, transformations \u2192 prefix with ~
4. Identify numeric values, measurements, magnitudes \u2192 prefix with $
5. Detect causal relationships \u2192 use \u2192 or \u2190
6. Detect logical connectives (and/or/therefore/because) \u2192 use \u2227 \u2228 \u2234 \u2235
7. Detect negation (not, no evidence, without, absence) \u2192 use \xAC or \u2205
8. Detect universal/existential quantifiers (all, every, some) \u2192 use \u2200 \u2203
9. Extract confidence level from hedge words \u2192 append confidence marker
10. Extract time references \u2192 append temporal marker
11. Strip filler words, articles, copulas, and social pleasantries
12. Hyphenate multi-word tokens: "climate change" \u2192 #climate-change

### Decoding Rules (AXON \u2192 Natural Language)

1. @ tokens \u2192 named entities
2. # tokens \u2192 concepts or abstract nouns
3. ~ tokens \u2192 verbs (conjugate naturally for context)
4. $ tokens \u2192 numeric values or scalar descriptors
5. \u2192 reads as "causes" or "leads to"
6. \u2190 reads as "is caused by" or "results from"
7. \u2234 reads as "therefore" \xB7 \u2235 reads as "because"
8. \xAC reads as "not" \xB7 \u2205 reads as "no [noun]" or "absence of"
9. \u2200 reads as "all" or "every" \xB7 \u2203 reads as "there exists"
10. \u2227 reads as "and" \xB7 \u2228 reads as "or" \xB7 \u2295 reads as "either...or (but not both)"
11. Confidence markers \u2192 hedge words (!! = "certainly", * = "probably", ** = "possibly")
12. Temporal markers \u2192 time phrases (^now = "currently", ^T+30d = "in 30 days")

### Examples

  "The sun probably emits ultraviolet radiation."
  \u2192 @sun ~emit* #UV-radiation

  "All living things require energy to survive."
  \u2192 \u2200@organism \u2283 (#energy \u2192 #survival!!)

  "Climate change caused by CO2 leads to temperature rise."
  \u2192 @CO2-emission \u2192 #climate-change!! \u2192 \u0394$temp\u2191

  "There is no evidence that this treatment works."
  \u2192 \u2205evidence \u2234 \xAC~work(#treatment)

  "fix the bug in the auth service"
  \u2192 >fix bug:auth-service

  "what is the best way to cache"
  \u2192 ?best cache

  "add a field email to User"
  \u2192 @user+.email

## Behavior

The user's message has been translated from natural language into AXON notation to save tokens. Decode the AXON back to understand the intent, then respond naturally and helpfully as a coding assistant.`;
var ClaudeTerminalBridge = class {
  terminal;
  writeEmitter = new vscode.EventEmitter();
  closeEmitter = new vscode.EventEmitter();
  initialized = false;
  busy = false;
  /** Ensure the AXON Claude terminal exists and return it. */
  ensureTerminal() {
    if (this.terminal && vscode.window.terminals.includes(this.terminal)) {
      return this.terminal;
    }
    const writeEmitter = this.writeEmitter;
    const closeEmitter = this.closeEmitter;
    const self = this;
    const pty = {
      onDidWrite: writeEmitter.event,
      onDidClose: closeEmitter.event,
      open: () => {
        writeEmitter.fire("\x1B[1mAXON Claude Terminal\x1B[0m\r\n");
        writeEmitter.fire("\x1B[90mMessages from the AXON chat are sent here.\x1B[0m\r\n\r\n");
      },
      close: () => {
        self.terminal = void 0;
        self.initialized = false;
      },
      handleInput: () => {
      }
    };
    this.terminal = vscode.window.createTerminal({
      name: "AXON Claude",
      pty,
      iconPath: new vscode.ThemeIcon("sparkle")
    });
    return this.terminal;
  }
  /**
   * Send a prompt to `claude -p` and stream output back via `onChunk`.
   * Returns the full response text when the process exits.
   */
  async sendRequest(text, systemPrompt, onChunk) {
    if (this.busy) {
      return "Claude is still processing the previous request. Please wait.";
    }
    this.busy = true;
    this.ensureTerminal();
    const args = ["-p", "--output-format", "text"];
    if (systemPrompt && !this.initialized) {
      args.push("--system-prompt", systemPrompt);
    }
    args.push(text);
    const displayText = text.length > 120 ? text.substring(0, 120) + "\u2026" : text;
    this.writeEmitter.fire(`\x1B[36m\u276F\x1B[0m ${displayText}\r
`);
    try {
      return await new Promise((resolve) => {
        const proc = (0, import_child_process.spawn)("claude", args, {
          stdio: ["ignore", "pipe", "pipe"],
          cwd: vscode.workspace.workspaceFolders?.[0]?.uri.fsPath,
          env: { ...process.env }
        });
        let fullOutput = "";
        proc.stdout.on("data", (data) => {
          const chunk = data.toString();
          fullOutput += chunk;
          this.writeEmitter.fire(chunk.replace(/\n/g, "\r\n"));
          onChunk(fullOutput);
        });
        proc.stderr.on("data", (data) => {
          const chunk = data.toString();
          this.writeEmitter.fire(`\x1B[31m${chunk.replace(/\n/g, "\r\n")}\x1B[0m`);
        });
        proc.on("close", (code) => {
          this.writeEmitter.fire("\r\n");
          this.busy = false;
          if (code === 0) {
            this.initialized = true;
            resolve(fullOutput.trim());
          } else {
            resolve(fullOutput.trim() || `Claude exited with code ${code}`);
          }
        });
        proc.on("error", (err) => {
          const msg = `Failed to start claude: ${err.message}`;
          this.writeEmitter.fire(`\x1B[31m${msg}\x1B[0m\r
`);
          this.busy = false;
          resolve(msg);
        });
      });
    } catch (err) {
      this.busy = false;
      return `Error: ${err?.message || "Unknown error"}`;
    }
  }
  /** Clean up resources. */
  dispose() {
    this.writeEmitter.dispose();
    this.closeEmitter.dispose();
    this.terminal?.dispose();
  }
};
function activate(context) {
  const translator = getTranslator(context.extensionPath);
  const history = new ChatHistory(context.globalStorageUri.fsPath);
  const claudeBridge = new ClaudeTerminalBridge();
  context.subscriptions.push({ dispose: () => claudeBridge.dispose() });
  const chat2 = vscode.chat.createChatParticipant(
    "axon.chat",
    async (request, _context, stream, token) => {
      const userInput = request.prompt;
      if (!userInput.trim())
        return;
      const result = translator.translate(userInput);
      const axon = result.axon;
      const savings = result.savings;
      result.free();
      stream.markdown(
        `> **AXON** (\`${savings}%\` token savings): \`${axon}\`

`
      );
      const models = await vscode.lm.selectChatModels({
        vendor: "copilot"
      });
      let model = models[0];
      if (!model) {
        const allModels = await vscode.lm.selectChatModels();
        model = allModels[0];
      }
      if (!model) {
        const status = await sendToClaude(axon);
        stream.markdown(
          `> **AXON** \u2192 Claude Code fallback (no VS Code language model found)

*${status}*`
        );
        return;
      }
      const messages = [
        vscode.LanguageModelChatMessage.User(AXON_SYSTEM_PROMPT),
        vscode.LanguageModelChatMessage.User(axon)
      ];
      try {
        const chatResponse = await model.sendRequest(messages, {}, token);
        for await (const fragment of chatResponse.text) {
          stream.markdown(fragment);
        }
      } catch (err) {
        if (err?.code === "NoPermissions") {
          stream.markdown(
            `*Permission denied.* Click **Allow** when prompted to let AXON use the language model.

AXON translation: \`${axon}\``
          );
        } else {
          stream.markdown(
            `*Model unavailable.* Sending AXON to Claude Code instead...

\`${axon}\``
          );
          const status = await sendToClaude(axon);
          stream.markdown(`

*${status}*`);
        }
      }
    }
  );
  chat2.iconPath = vscode.Uri.joinPath(
    context.extensionUri,
    "resources",
    "axon.svg"
  );
  context.subscriptions.push(chat2);
  const sidebarProvider = new AxonSidebarProvider(translator, history, claudeBridge);
  context.subscriptions.push(
    vscode.window.registerWebviewViewProvider("axon.chatView", sidebarProvider)
  );
  context.subscriptions.push(
    vscode.commands.registerCommand("axon.openChat", async () => {
      await vscode.commands.executeCommand("axon.chatView.focus");
    })
  );
  let editorPanel;
  context.subscriptions.push(
    vscode.commands.registerCommand("axon.openInNewTab", () => {
      if (editorPanel) {
        editorPanel.reveal();
        return;
      }
      editorPanel = vscode.window.createWebviewPanel(
        "axon.chatTab",
        "AXON Chat",
        vscode.ViewColumn.One,
        { enableScripts: true, retainContextWhenHidden: true }
      );
      editorPanel.iconPath = vscode.Uri.joinPath(
        context.extensionUri,
        "resources",
        "axon.svg"
      );
      editorPanel.webview.html = getChatHtml();
      wireUpWebview(editorPanel.webview, translator, history, claudeBridge);
      editorPanel.onDidDispose(() => {
        editorPanel = void 0;
      });
    })
  );
  context.subscriptions.push(
    vscode.commands.registerCommand("axon.translate", async () => {
      const input = await getInput();
      if (!input)
        return;
      const result = translator.translate(input);
      const pick = await vscode.window.showQuickPick(
        [
          {
            label: `$(sparkle) ${result.axon}`,
            description: `${result.savings}% token savings`,
            detail: result.annotation,
            target: "preview"
          },
          {
            label: "$(comment-discussion) Open in AXON Chat",
            target: "chat"
          },
          { label: "$(terminal) Send to Claude Code", target: "claudeCode" },
          {
            label: "$(comment-discussion) Send to Copilot Chat",
            target: "copilot"
          },
          { label: "$(clippy) Copy to Clipboard", target: "clipboard" }
        ],
        {
          title: "AXON Translation",
          placeHolder: `${result.axon}  (${result.savings}% savings)`
        }
      );
      if (!pick || pick.target === "preview")
        return;
      if (pick.target === "chat") {
        await vscode.commands.executeCommand("workbench.action.chat.open", {
          query: `@axon ${input}`
        });
      } else {
        await dispatch(pick.target, result.axon, result.savings);
      }
    })
  );
  context.subscriptions.push(
    vscode.commands.registerCommand("axon.translateToClaudeCode", async () => {
      const input = await getInput();
      if (!input)
        return;
      const result = translator.translate(input);
      await dispatch("claudeCode", result.axon, result.savings);
    })
  );
  context.subscriptions.push(
    vscode.commands.registerCommand("axon.translateToCopilot", async () => {
      const input = await getInput();
      if (!input)
        return;
      const result = translator.translate(input);
      await dispatch("copilot", result.axon, result.savings);
    })
  );
  context.subscriptions.push(
    vscode.commands.registerCommand("axon.translateToClipboard", async () => {
      const input = await getInput();
      if (!input)
        return;
      const result = translator.translate(input);
      await dispatch("clipboard", result.axon, result.savings);
    })
  );
}
async function getInput() {
  const editor = vscode.window.activeTextEditor;
  const selection = editor?.selection;
  if (selection && !selection.isEmpty) {
    return editor.document.getText(selection);
  }
  return vscode.window.showInputBox({
    prompt: "Enter natural language to translate to AXON",
    placeHolder: "e.g. fix the bug in the auth service"
  });
}
async function dispatch(target, axon, savings) {
  switch (target) {
    case "claudeCode": {
      const status = await sendToClaude(axon);
      vscode.window.showInformationMessage(
        `${status} (${savings}% savings)`
      );
      break;
    }
    case "copilot": {
      try {
        await vscode.commands.executeCommand("workbench.action.chat.open", {
          query: axon
        });
      } catch {
        await vscode.env.clipboard.writeText(axon);
        vscode.window.showWarningMessage(
          "Copilot Chat not available. AXON copied to clipboard instead."
        );
      }
      break;
    }
    case "clipboard":
    default: {
      await vscode.env.clipboard.writeText(axon);
      vscode.window.showInformationMessage(
        `AXON copied to clipboard (${savings}% savings)`
      );
      break;
    }
  }
}
var terminalSpecInitialized = false;
async function sendToClaudeWindow(text) {
  try {
    await vscode.commands.executeCommand(
      "claude-vscode.editor.open",
      void 0,
      text
    );
    return true;
  } catch {
    return false;
  }
}
async function sendToClaudeTerminal(text) {
  const args = ["-p", "--output-format", "text"];
  if (!terminalSpecInitialized) {
    args.push("--system-prompt", AXON_SYSTEM_PROMPT);
  }
  args.push(text);
  return new Promise((resolve) => {
    const proc = (0, import_child_process.spawn)("claude", args, {
      stdio: ["ignore", "pipe", "pipe"],
      cwd: vscode.workspace.workspaceFolders?.[0]?.uri.fsPath,
      env: { ...process.env }
    });
    let output = "";
    proc.stdout.on("data", (data) => {
      output += data.toString();
    });
    proc.stderr.on("data", (_data) => {
    });
    proc.on("close", (code) => {
      if (code === 0) {
        terminalSpecInitialized = true;
      }
      resolve(output.trim() || `Claude exited with code ${code}`);
    });
    proc.on("error", (err) => {
      resolve(`Failed to start claude: ${err.message}`);
    });
  });
}
async function sendToClaude(text, _webview) {
  if (await sendToClaudeWindow(text)) {
    return "AXON sent to Claude Code.";
  }
  return await sendToClaudeTerminal(text);
}
function wireUpWebview(webview, translator, history, bridge) {
  webview.onDidReceiveMessage(async (msg) => {
    if (msg.type === "init") {
      const target = msg.target;
      if (target === "claude") {
        const response = await bridge.sendRequest(
          "Acknowledge that you understand the AXON notation system and are ready to receive AXON-encoded messages.",
          AXON_SYSTEM_PROMPT,
          (chunk) => webview.postMessage({ type: "response-stream", text: chunk })
        );
        webview.postMessage({ type: "response-stream", text: response });
        webview.postMessage({ type: "response-done" });
      } else if (target === "copilot") {
        try {
          const models = await vscode.lm.selectChatModels({ vendor: "copilot" });
          let model = models[0];
          if (!model) {
            const allModels = await vscode.lm.selectChatModels();
            model = allModels[0];
          }
          if (!model) {
            webview.postMessage({ type: "response-stream", text: "No language model available. Install GitHub Copilot or another LM extension." });
            webview.postMessage({ type: "response-done" });
            return;
          }
          const initPrompt = AXON_SYSTEM_PROMPT;
          const messages = [
            vscode.LanguageModelChatMessage.User(
              initPrompt + "\n\nAcknowledge that you understand the AXON notation system and are ready to receive AXON-encoded messages."
            )
          ];
          const chatResponse = await model.sendRequest(messages, {});
          let fullResponse = "";
          for await (const fragment of chatResponse.text) {
            fullResponse += fragment;
            webview.postMessage({ type: "response-stream", text: fullResponse });
          }
          webview.postMessage({ type: "response-done" });
        } catch (err) {
          const errMsg = err?.code === "NoPermissions" ? "Permission denied. Click Allow when prompted." : `Error: ${err?.message || "Unknown error"}`;
          webview.postMessage({ type: "response-stream", text: errMsg });
          webview.postMessage({ type: "response-done" });
        }
      }
      return;
    }
    if (msg.type === "history") {
      const query = msg.query || "";
      const results = query.trim() ? history.search(query) : history.recent(50);
      webview.postMessage({ type: "history-results", results, query, _explicit: !!msg._explicit });
      return;
    }
    if (msg.type === "send") {
      const input = msg.text;
      if (!input?.trim())
        return;
      const result = translator.translate(input);
      const axon = result.axon;
      const savings = result.savings;
      result.free();
      const entry = history.add({ userInput: input, axon, savings, response: "" });
      webview.postMessage({ type: "axon", axon, savings });
      try {
        const response = await bridge.sendRequest(
          axon,
          AXON_SYSTEM_PROMPT,
          (chunk) => webview.postMessage({ type: "response-stream", text: chunk })
        );
        history.updateResponse(entry.id, response);
        webview.postMessage({ type: "response-done" });
      } catch (err) {
        const errMsg = `*Error:* ${err?.message || "Unknown error"}`;
        history.updateResponse(entry.id, errMsg);
        webview.postMessage({ type: "response-stream", text: errMsg });
        webview.postMessage({ type: "response-done" });
      }
    }
  });
}
var AxonSidebarProvider = class {
  constructor(translator, history, bridge) {
    this.translator = translator;
    this.history = history;
    this.bridge = bridge;
  }
  resolveWebviewView(webviewView, _context, _token) {
    webviewView.webview.options = { enableScripts: true };
    webviewView.webview.html = getChatHtml();
    wireUpWebview(webviewView.webview, this.translator, this.history, this.bridge);
  }
};
function getChatHtml() {
  return `<!DOCTYPE html>
<html>
<head>
<style>
  * { box-sizing: border-box; margin: 0; padding: 0; }
  body {
    font-family: var(--vscode-font-family, system-ui, sans-serif);
    font-size: var(--vscode-font-size, 13px);
    color: var(--vscode-foreground);
    background: var(--vscode-sideBar-background);
    padding: 8px;
    display: flex;
    flex-direction: column;
    height: 100vh;
  }
  .chat-header {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 10px;
    padding: 16px 0 12px;
    border-bottom: 1px solid var(--vscode-panel-border);
    margin-bottom: 8px;
  }
  .chat-logo {
    font-family: var(--vscode-editor-font-family, monospace);
    font-size: 24px;
    font-weight: 600;
    letter-spacing: 0.08em;
    color: var(--vscode-foreground);
  }
  .chat-logo span {
    color: var(--vscode-textLink-foreground);
  }
  .session-select {
    width: 100%;
    max-width: 240px;
    appearance: none;
    -webkit-appearance: none;
    background: var(--vscode-input-background);
    color: var(--vscode-input-foreground);
    border: 1px solid var(--vscode-input-border, transparent);
    border-radius: 4px;
    padding: 5px 28px 5px 10px;
    font-family: inherit;
    font-size: 12px;
    cursor: pointer;
    outline: none;
    background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='10' height='6'%3E%3Cpath d='M0 0l5 6 5-6z' fill='%23888'/%3E%3C/svg%3E");
    background-repeat: no-repeat;
    background-position: right 8px center;
  }
  .session-select:focus {
    border-color: var(--vscode-focusBorder);
  }
  #messages {
    flex: 1;
    overflow-y: auto;
    padding-bottom: 8px;
  }
  .msg {
    margin-bottom: 12px;
    line-height: 1.5;
  }
  .msg-user {
    color: var(--vscode-foreground);
    opacity: 0.7;
    font-size: 12px;
  }
  .msg-axon {
    font-family: var(--vscode-editor-font-family, monospace);
    color: var(--vscode-textLink-foreground);
    font-size: 13px;
    font-weight: 600;
    padding: 4px 0;
  }
  .msg-savings {
    font-size: 11px;
    color: var(--vscode-charts-green, #4ade80);
    margin-bottom: 4px;
  }
  .msg-response {
    word-wrap: break-word;
    line-height: 1.5;
  }
  .msg-response p {
    margin: 0.4em 0;
  }
  .msg-response code {
    background: var(--vscode-textCodeBlock-background);
    padding: 1px 4px;
    border-radius: 3px;
    font-family: var(--vscode-editor-font-family, monospace);
    font-size: 12px;
  }
  .msg-response pre {
    background: var(--vscode-textCodeBlock-background);
    padding: 8px 10px;
    border-radius: 4px;
    overflow-x: auto;
    margin: 0.5em 0;
  }
  .msg-response pre code {
    background: none;
    padding: 0;
    font-size: 12px;
  }
  .msg-response h1, .msg-response h2, .msg-response h3,
  .msg-response h4, .msg-response h5, .msg-response h6 {
    margin: 0.6em 0 0.3em;
    font-weight: 600;
  }
  .msg-response h1 { font-size: 1.3em; }
  .msg-response h2 { font-size: 1.15em; }
  .msg-response h3 { font-size: 1.05em; }
  .msg-response ul, .msg-response ol {
    margin: 0.3em 0;
    padding-left: 1.5em;
  }
  .msg-response li {
    margin: 0.15em 0;
  }
  .msg-response blockquote {
    border-left: 3px solid var(--vscode-textBlockQuote-border, #555);
    padding: 2px 10px;
    margin: 0.4em 0;
    opacity: 0.85;
  }
  .msg-response hr {
    border: none;
    border-top: 1px solid var(--vscode-panel-border);
    margin: 0.6em 0;
  }
  .msg-response a {
    color: var(--vscode-textLink-foreground);
  }
  .msg-response table {
    border-collapse: collapse;
    margin: 0.4em 0;
  }
  .msg-response th, .msg-response td {
    border: 1px solid var(--vscode-panel-border);
    padding: 4px 8px;
    text-align: left;
  }
  .msg-response th {
    background: var(--vscode-textCodeBlock-background);
    font-weight: 600;
  }
  #input-area {
    display: flex;
    gap: 4px;
    padding-top: 8px;
    border-top: 1px solid var(--vscode-panel-border);
  }
  #input {
    flex: 1;
    background: var(--vscode-input-background);
    color: var(--vscode-input-foreground);
    border: 1px solid var(--vscode-input-border, transparent);
    padding: 6px 8px;
    border-radius: 4px;
    font-family: inherit;
    font-size: 13px;
    outline: none;
    resize: none;
  }
  #input:focus {
    border-color: var(--vscode-focusBorder);
  }
  #input::placeholder {
    color: var(--vscode-input-placeholderForeground);
  }
  button {
    background: var(--vscode-button-background);
    color: var(--vscode-button-foreground);
    border: none;
    padding: 6px 12px;
    border-radius: 4px;
    cursor: pointer;
    font-size: 12px;
    font-weight: 600;
    white-space: nowrap;
  }
  button:hover {
    background: var(--vscode-button-hoverBackground);
  }
  .spinner {
    display: inline-block;
    width: 12px;
    height: 12px;
    border: 2px solid var(--vscode-foreground);
    border-top-color: transparent;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
    margin-right: 6px;
    vertical-align: middle;
    opacity: 0.5;
  }
  @keyframes spin { to { transform: rotate(360deg); } }
  .history-header {
    font-size: 12px;
    font-weight: 600;
    color: var(--vscode-foreground);
    opacity: 0.6;
    margin-bottom: 8px;
    padding-bottom: 4px;
    border-bottom: 1px solid var(--vscode-panel-border);
  }
  .history-entry {
    margin-bottom: 10px;
    padding: 6px 8px;
    border-radius: 4px;
    background: var(--vscode-editor-background);
    border: 1px solid var(--vscode-panel-border);
    font-size: 12px;
  }
  .history-entry .he-time {
    font-size: 10px;
    opacity: 0.5;
  }
  .history-entry .he-user {
    opacity: 0.7;
    margin: 2px 0;
  }
  .history-entry .he-axon {
    font-family: var(--vscode-editor-font-family, monospace);
    color: var(--vscode-textLink-foreground);
    font-weight: 600;
  }
  .history-entry .he-response {
    word-wrap: break-word;
    margin-top: 4px;
    opacity: 0.85;
  }
  .history-close {
    font-size: 11px;
    opacity: 0.6;
    cursor: pointer;
    text-decoration: underline;
    margin-top: 4px;
    display: inline-block;
  }
  .history-close:hover { opacity: 1; }
</style>
</head>
<body>
  <div class="chat-header">
    <div class="chat-logo">AX<span>ON</span></div>
    <select class="session-select" id="session-select" onchange="loadSession(this.value)">
      <option value="current">Current session</option>
    </select>
  </div>
  <div id="messages">
    <div class="msg" style="opacity:0.4;font-size:12px;padding:16px 0;">
      Type a question in natural language. AXON translates it to save tokens, then sends it to the AI.<br><br>
      <strong>Slash commands:</strong><br>
      <code>/clear</code> \u2014 clear chat<br>
      <code>/init claude</code> \u2014 initialise context with Claude<br>
      <code>/init copilot</code> \u2014 initialise context with Copilot<br>
      <code>/history</code> \u2014 recent chat history<br>
      <code>/history &lt;query&gt;</code> \u2014 search chat history
    </div>
  </div>
  <div id="input-area">
    <textarea id="input" rows="1" placeholder="Ask something... (/clear, /init, /history)"></textarea>
    <button id="send" onclick="send()">Send</button>
  </div>
  <script>
    const vscode = acquireVsCodeApi();
    const messagesEl = document.getElementById('messages');
    const inputEl = document.getElementById('input');
    let currentResponseEl = null;

    function send() {
      const text = inputEl.value.trim();
      if (!text) return;

      // Handle slash commands
      if (text === '/clear') {
        messagesEl.innerHTML = '';
        inputEl.value = '';
        inputEl.style.height = 'auto';
        return;
      }

      const historyMatch = text.match(/^\\/history(?:\\s+(.+))?$/i);
      if (historyMatch) {
        const query = (historyMatch[1] || '').trim();
        vscode.postMessage({ type: 'history', query, _explicit: true });
        inputEl.value = '';
        inputEl.style.height = 'auto';
        return;
      }

      const initMatch = text.match(/^\\/init\\s+(claude|copilot)$/i);
      if (initMatch) {
        const target = initMatch[1].toLowerCase();
        const userDiv = document.createElement('div');
        userDiv.className = 'msg msg-user';
        userDiv.textContent = text;
        messagesEl.appendChild(userDiv);

        currentResponseEl = document.createElement('div');
        currentResponseEl.className = 'msg';
        currentResponseEl.innerHTML = '<span class="spinner"></span> Initialising context with ' + escapeHtml(target) + '...';
        messagesEl.appendChild(currentResponseEl);
        messagesEl.scrollTop = messagesEl.scrollHeight;

        vscode.postMessage({ type: 'init', target });
        inputEl.value = '';
        inputEl.style.height = 'auto';
        return;
      }

      // Show user message
      const userDiv = document.createElement('div');
      userDiv.className = 'msg msg-user';
      userDiv.textContent = text;
      messagesEl.appendChild(userDiv);

      // Create placeholder for AXON + response
      currentResponseEl = document.createElement('div');
      currentResponseEl.className = 'msg';
      currentResponseEl.innerHTML = '<span class="spinner"></span> Translating...';
      messagesEl.appendChild(currentResponseEl);

      messagesEl.scrollTop = messagesEl.scrollHeight;

      vscode.postMessage({ type: 'send', text });
      inputEl.value = '';
      inputEl.style.height = 'auto';
    }

    inputEl.addEventListener('keydown', (e) => {
      if (e.key === 'Enter' && !e.shiftKey) {
        e.preventDefault();
        send();
      }
    });

    // Auto-resize textarea
    inputEl.addEventListener('input', () => {
      inputEl.style.height = 'auto';
      inputEl.style.height = Math.min(inputEl.scrollHeight, 120) + 'px';
    });

    window.addEventListener('message', (e) => {
      const msg = e.data;

      if (msg.type === 'history-results') {
        // Always update session dropdown with latest results
        populateSessions(msg.results);

        // If this was a background request (no query, on init), don't show panel
        if (!msg.query && !msg._explicit) return;

        // Remove any previous history panel
        const prev = document.getElementById('history-panel');
        if (prev) prev.remove();

        const panel = document.createElement('div');
        panel.id = 'history-panel';
        panel.className = 'msg';

        const header = msg.query
          ? 'Search results for "' + escapeHtml(msg.query) + '"'
          : 'Recent chat history';
        let html = '<div class="history-header">' + header + '</div>';

        if (msg.results.length === 0) {
          html += '<div style="opacity:0.5;font-size:12px;">No results found.</div>';
        } else {
          for (const entry of msg.results) {
            const date = new Date(entry.timestamp);
            const timeStr = date.toLocaleDateString() + ' ' + date.toLocaleTimeString([], {hour:'2-digit',minute:'2-digit'});
            html += '<div class="history-entry">'
              + '<div class="he-time">' + escapeHtml(timeStr) + '</div>'
              + '<div class="he-user">' + escapeHtml(entry.userInput) + '</div>'
              + '<div class="he-axon">' + escapeHtml(entry.axon) + ' <span class="msg-savings" style="display:inline;">' + entry.savings + '%</span></div>'
              + (entry.response ? '<div class="he-response">' + renderMarkdown(entry.response.substring(0, 200)) + (entry.response.length > 200 ? '...' : '') + '</div>' : '')
              + '</div>';
          }
        }
        html += '<span class="history-close" onclick="this.parentElement.remove()">Close history</span>';
        panel.innerHTML = html;
        messagesEl.appendChild(panel);
        messagesEl.scrollTop = messagesEl.scrollHeight;
        return;
      }

      if (!currentResponseEl) return;

      if (msg.type === 'axon') {
        currentResponseEl.innerHTML =
          '<div class="msg-axon">' + escapeHtml(msg.axon) + '</div>' +
          '<div class="msg-savings">' + msg.savings + '% token savings</div>' +
          '<div class="msg-response"><span class="spinner"></span></div>';
      } else if (msg.type === 'response-stream') {
        const respEl = currentResponseEl.querySelector('.msg-response');
        if (respEl) respEl.innerHTML = renderMarkdown(msg.text);
      } else if (msg.type === 'response-done') {
        const spinner = currentResponseEl.querySelector('.spinner');
        if (spinner) spinner.remove();
        currentResponseEl = null;
      }

      messagesEl.scrollTop = messagesEl.scrollHeight;
    });

    function escapeHtml(s) {
      return String(s).replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;');
    }

    function renderMarkdown(src) {
      var e = escapeHtml;
      // Extract fenced code blocks first
      var blocks = [];
      src = src.replace(/\`\`\`(\\w*)\\n([\\s\\S]*?)\`\`\`/g, function(_, lang, code) {
        blocks.push('<pre><code' + (lang ? ' class="language-' + e(lang) + '"' : '') + '>' + e(code.replace(/\\n$/, '')) + '</code></pre>');
        return '\\x00BLOCK' + (blocks.length - 1) + '\\x00';
      });
      // Split into lines for block-level processing
      var lines = src.split('\\n');
      var html = '';
      var inList = false;
      var listType = '';
      for (var i = 0; i < lines.length; i++) {
        var line = lines[i];
        // Block placeholder
        var bm = line.match(/^\\x00BLOCK(\\d+)\\x00$/);
        if (bm) {
          if (inList) { html += '</' + listType + '>'; inList = false; }
          html += blocks[parseInt(bm[1])];
          continue;
        }
        // Headings
        var hm = line.match(/^(#{1,6})\\s+(.+)$/);
        if (hm) {
          if (inList) { html += '</' + listType + '>'; inList = false; }
          var lvl = hm[1].length;
          html += '<h' + lvl + '>' + inlineMarkdown(e(hm[2])) + '</h' + lvl + '>';
          continue;
        }
        // Horizontal rule
        if (/^(\\*{3,}|-{3,}|_{3,})$/.test(line.trim())) {
          if (inList) { html += '</' + listType + '>'; inList = false; }
          html += '<hr>';
          continue;
        }
        // Blockquote
        var bq = line.match(/^>\\s?(.*)$/);
        if (bq) {
          if (inList) { html += '</' + listType + '>'; inList = false; }
          html += '<blockquote>' + inlineMarkdown(e(bq[1])) + '</blockquote>';
          continue;
        }
        // Unordered list
        var ul = line.match(/^\\s*[-*+]\\s+(.+)$/);
        if (ul) {
          if (!inList || listType !== 'ul') {
            if (inList) html += '</' + listType + '>';
            html += '<ul>'; inList = true; listType = 'ul';
          }
          html += '<li>' + inlineMarkdown(e(ul[1])) + '</li>';
          continue;
        }
        // Ordered list
        var ol = line.match(/^\\s*\\d+\\.\\s+(.+)$/);
        if (ol) {
          if (!inList || listType !== 'ol') {
            if (inList) html += '</' + listType + '>';
            html += '<ol>'; inList = true; listType = 'ol';
          }
          html += '<li>' + inlineMarkdown(e(ol[1])) + '</li>';
          continue;
        }
        // Close list if open
        if (inList && line.trim() === '') {
          html += '</' + listType + '>'; inList = false;
        }
        // Empty line = paragraph break
        if (line.trim() === '') {
          html += '<br>';
          continue;
        }
        // Table row
        if (line.indexOf('|') !== -1 && line.trim().startsWith('|')) {
          if (inList) { html += '</' + listType + '>'; inList = false; }
          // Skip separator rows
          if (/^[\\s|:-]+$/.test(line)) continue;
          var cells = line.split('|').filter(function(c,idx,arr) { return idx > 0 && idx < arr.length - 1; });
          var isHeader = i + 1 < lines.length && /^[\\s|:-]+$/.test(lines[i+1]);
          var tag = isHeader ? 'th' : 'td';
          html += '<table><tr>' + cells.map(function(c) { return '<' + tag + '>' + inlineMarkdown(e(c.trim())) + '</' + tag + '>'; }).join('') + '</tr></table>';
          continue;
        }
        // Regular paragraph
        if (inList) { html += '</' + listType + '>'; inList = false; }
        html += '<p>' + inlineMarkdown(e(line)) + '</p>';
      }
      if (inList) html += '</' + listType + '>';
      return html;
    }

    function inlineMarkdown(s) {
      // Inline code
      s = s.replace(/\`([^\`]+)\`/g, '<code>$1</code>');
      // Bold+italic
      s = s.replace(/\\*\\*\\*(.+?)\\*\\*\\*/g, '<strong><em>$1</em></strong>');
      // Bold
      s = s.replace(/\\*\\*(.+?)\\*\\*/g, '<strong>$1</strong>');
      // Italic
      s = s.replace(/\\*(.+?)\\*/g, '<em>$1</em>');
      // Strikethrough
      s = s.replace(/~~(.+?)~~/g, '<del>$1</del>');
      // Links
      s = s.replace(/\\[([^\\]]+)\\]\\(([^)]+)\\)/g, '<a href="$2">$1</a>');
      return s;
    }

    // \u2500\u2500 Session management \u2500\u2500
    let sessions = [];
    const sessionSelect = document.getElementById('session-select');

    function populateSessions(results) {
      sessions = results || [];
      // Group entries by day
      const grouped = {};
      for (const entry of sessions) {
        const date = new Date(entry.timestamp);
        const key = date.toLocaleDateString();
        if (!grouped[key]) grouped[key] = [];
        grouped[key].push(entry);
      }

      sessionSelect.innerHTML = '<option value="current">Current session</option>';
      const days = Object.keys(grouped).reverse();
      for (const day of days) {
        const entries = grouped[day];
        const first = entries[0];
        const label = day + ' (' + entries.length + ' message' + (entries.length > 1 ? 's' : '') + ')';
        const opt = document.createElement('option');
        opt.value = day;
        opt.textContent = label;
        sessionSelect.appendChild(opt);
      }
    }

    window.loadSession = function(value) {
      if (value === 'current') {
        // Restore current session view
        messagesEl.innerHTML = '';
        return;
      }
      // Show entries from the selected day
      const grouped = {};
      for (const entry of sessions) {
        const date = new Date(entry.timestamp);
        const key = date.toLocaleDateString();
        if (!grouped[key]) grouped[key] = [];
        grouped[key].push(entry);
      }
      const entries = grouped[value] || [];
      messagesEl.innerHTML = '';
      for (const entry of entries) {
        const time = new Date(entry.timestamp).toLocaleTimeString([], {hour:'2-digit',minute:'2-digit'});
        const div = document.createElement('div');
        div.className = 'msg';
        div.innerHTML =
          '<div class="msg-user">' + escapeHtml(entry.userInput) + ' <span style="opacity:0.4;font-size:10px;">' + escapeHtml(time) + '</span></div>' +
          '<div class="msg-axon">' + escapeHtml(entry.axon) + '</div>' +
          '<div class="msg-savings">' + entry.savings + '% token savings</div>' +
          (entry.response ? '<div class="msg-response">' + renderMarkdown(entry.response) + '</div>' : '');
        messagesEl.appendChild(div);
      }
      messagesEl.scrollTop = 0;
    };

    // Request history on load to populate session dropdown
    vscode.postMessage({ type: 'history', query: '' });

  </script>
</body>
</html>`;
}
function deactivate() {
}
// Annotate the CommonJS export names for ESM import in node:
0 && (module.exports = {
  activate,
  deactivate
});
//# sourceMappingURL=extension.js.map
