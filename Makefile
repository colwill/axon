.PHONY: build serve serve-playground build-repl run-repl build-vscode rebuild-vscode package-vscode install-vscode

build:
	wasm-pack build --target web --out-dir playground/pkg

build-repl:
	cargo build --release -p axon-frontend

run-repl:
	cargo run -p axon-frontend

serve: build
	python3 -m http.server 3001 --directory playground

serve-playground: build
	python3 -m http.server 3002

build-vscode:
	wasm-pack build --target nodejs --out-dir vscode-extension/wasm
	cd vscode-extension && npm install && npm run build

rebuild-vscode:
	cd vscode-extension && npm run build

package-vscode: build-vscode
	cd vscode-extension && npm run package

install-vscode: rebuild-vscode
	cd vscode-extension && npx @vscode/vsce package --no-dependencies && \
		code --install-extension $$(ls -t *.vsix | head -1) --force
