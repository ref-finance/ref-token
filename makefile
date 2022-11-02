RFLAGS="-C link-arg=-s"

build: xref-token tt
	rustup target add wasm32-unknown-unknown
	RUSTFLAGS=$(RFLAGS) cargo build -p xref-token --target wasm32-unknown-unknown --release
	mkdir -p res
	cp target/wasm32-unknown-unknown/release/xref_token.wasm ./res/xref_token.wasm

tt: test-token
	rustup target add wasm32-unknown-unknown
	RUSTFLAGS=$(RFLAGS) cargo build -p test-token --target wasm32-unknown-unknown --release
	mkdir -p res
	cp target/wasm32-unknown-unknown/release/test_token.wasm ./res/test_token.wasm

release:
	$(call docker_build)
	mkdir -p res
	cp target/wasm32-unknown-unknown/release/xref_token.wasm res/xref_token_release.wasm

unittest: build
ifdef TC
	RUSTFLAGS=$(RFLAGS) cargo test $(TC) -p xref-token --lib -- --nocapture
else
	RUSTFLAGS=$(RFLAGS) cargo test -p xref-token --lib -- --nocapture
endif

test: build
ifdef TF
	RUSTFLAGS=$(RFLAGS) cargo test -p xref-token --test $(TF) -- --nocapture
else
	RUSTFLAGS=$(RFLAGS) cargo test -p xref-token --tests -- --nocapture
endif

clean:
	cargo clean
	rm -rf res/

define docker_build
	docker build -t my-xref-builder .
	docker run \
		--mount type=bind,source=${PWD},target=/host \
		--cap-add=SYS_PTRACE --security-opt seccomp=unconfined \
		-w /host \
		-e RUSTFLAGS=$(RFLAGS) \
		-i -t my-xref-builder \
		cargo build -p xref-token --target wasm32-unknown-unknown --release
endef