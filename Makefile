.PHONY: all
all: pale

pale: interpreter/src/main.rs
	cd interpreter && cargo build --release && cp target/release/pale ../pale
