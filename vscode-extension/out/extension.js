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
var AXON_SYSTEM_PROMPT = `You are fluent in AXON (AI eXchange Optimised Notation). The user's message has been translated from natural language into AXON notation to save tokens. Decode the AXON back to understand the intent, then respond naturally and helpfully.

AXON Quick Reference:
  @ entity/type  # concept  ~ process/function  > command  ? query  $ value  ^ temporal  . member  \u2205 null
  \u2192 causes/to  \u2190 from  : scope/type  = assign  + add  - remove  < extends
  >doc >impl >fix >test >rev >ref >opt >plan >dep >add >rm >up >mv >cfg >mig >db >api >ci >sec >err >log
  ?how ?why ?best ?what ?diff ?when ?where ?can ?cmp ?alt ?err ?perf
  @Type+.field (add field)  @Type-.field (remove)  @Type.x=$v (set)  @Type:impl(@Trait) (implement)

Respond to the decoded intent as a helpful coding assistant.`;
function activate(context) {
  const translator = getTranslator(context.extensionPath);
  const history = new ChatHistory(context.globalStorageUri.fsPath);
  context.subscriptions.push(
    vscode.window.onDidCloseTerminal((closed) => {
      if (closed === axonTerminal) {
        axonTerminal = void 0;
        terminalSpecInitialized = false;
      }
    })
  );
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
  const sidebarProvider = new AxonSidebarProvider(translator, history);
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
      wireUpWebview(editorPanel.webview, translator, history);
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
var axonTerminal;
var terminalSpecInitialized = false;
function findAxonTerminal() {
  if (axonTerminal) {
    if (vscode.window.terminals.includes(axonTerminal)) {
      return axonTerminal;
    }
    axonTerminal = void 0;
    terminalSpecInitialized = false;
  }
  const existing = vscode.window.terminals.find(
    (t) => t.name === "AXON Claude"
  );
  if (existing) {
    axonTerminal = existing;
    terminalSpecInitialized = false;
  }
  return existing;
}
function shellEscape(s) {
  return s.replace(/'/g, "'\\''");
}
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
function sendToClaudeTerminal(text) {
  let terminal = findAxonTerminal();
  if (!terminal) {
    terminal = vscode.window.createTerminal({
      name: "AXON Claude",
      iconPath: new vscode.ThemeIcon("sparkle")
    });
    axonTerminal = terminal;
    terminalSpecInitialized = false;
  }
  terminal.show(
    /* preserveFocus */
    false
  );
  const systemPromptArg = terminalSpecInitialized ? "" : ` --system-prompt '${shellEscape(AXON_SYSTEM_PROMPT)}'`;
  const cmd = `claude -p${systemPromptArg} '${shellEscape(text)}'`;
  terminal.sendText(cmd);
  terminalSpecInitialized = true;
  return "AXON sent to Claude CLI terminal.";
}
async function sendToClaude(text, _webview) {
  if (await sendToClaudeWindow(text)) {
    return "AXON sent to Claude Code.";
  }
  return sendToClaudeTerminal(text);
}
function wireUpWebview(webview, translator, history) {
  webview.onDidReceiveMessage(async (msg) => {
    if (msg.type === "init") {
      const target = msg.target;
      const initPrompt = AXON_SYSTEM_PROMPT;
      if (target === "claude") {
        const status = await sendToClaude(initPrompt, webview);
        webview.postMessage({ type: "response-stream", text: status });
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
        const models = await vscode.lm.selectChatModels({ vendor: "copilot" });
        let model = models[0];
        if (!model) {
          const allModels = await vscode.lm.selectChatModels();
          model = allModels[0];
        }
        if (model) {
          const messages = [
            vscode.LanguageModelChatMessage.User(AXON_SYSTEM_PROMPT),
            vscode.LanguageModelChatMessage.User(axon)
          ];
          const chatResponse = await model.sendRequest(messages, {});
          let fullResponse = "";
          for await (const fragment of chatResponse.text) {
            fullResponse += fragment;
            webview.postMessage({ type: "response-stream", text: fullResponse });
          }
          history.updateResponse(entry.id, fullResponse);
          webview.postMessage({ type: "response-done" });
        } else {
          const status = await sendToClaude(axon, webview);
          history.updateResponse(entry.id, status);
          webview.postMessage({ type: "response-stream", text: status });
          webview.postMessage({ type: "response-done" });
        }
      } catch (err) {
        const errMsg = err?.code === "NoPermissions" ? "*Permission denied.* Click Allow when prompted." : `*Error:* ${err?.message || "Unknown error"}`;
        history.updateResponse(entry.id, errMsg);
        webview.postMessage({ type: "response-stream", text: errMsg });
        webview.postMessage({ type: "response-done" });
      }
    }
  });
}
var AxonSidebarProvider = class {
  constructor(translator, history) {
    this.translator = translator;
    this.history = history;
  }
  resolveWebviewView(webviewView, _context, _token) {
    webviewView.webview.options = { enableScripts: true };
    webviewView.webview.html = getChatHtml();
    wireUpWebview(webviewView.webview, this.translator, this.history);
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
    white-space: pre-wrap;
    word-wrap: break-word;
  }
  .msg-response code {
    background: var(--vscode-textCodeBlock-background);
    padding: 1px 4px;
    border-radius: 3px;
    font-family: var(--vscode-editor-font-family, monospace);
    font-size: 12px;
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
    white-space: pre-wrap;
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
              + (entry.response ? '<div class="he-response">' + escapeHtml(entry.response.substring(0, 200)) + (entry.response.length > 200 ? '...' : '') + '</div>' : '')
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
        if (respEl) respEl.innerHTML = escapeHtml(msg.text);
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
          (entry.response ? '<div class="msg-response">' + escapeHtml(entry.response) + '</div>' : '');
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
