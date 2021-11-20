.PHONY: run
run: compile
	RUST_LOG=debug sudo -E ./target/debug/hyperion

.PHONY: compile
compile:
	cargo build $(CFLAGS)
