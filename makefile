RFLAGS="-C link-arg=-s"

build: xref-token
	rustup target add wasm32-unknown-unknown
	RUSTFLAGS=$(RFLAGS) cargo build -p xref-token --target wasm32-unknown-unknown --release
	mkdir -p res
	cp target/wasm32-unknown-unknown/release/xref_token.wasm ./res/xref_token.wasm
