.PHONY: all
all: pale

pale: $(shell find src -type f) $(shell find interpreter/src -type f)
	cd interpreter && cargo build --release && cp target/release/pale ../pale
