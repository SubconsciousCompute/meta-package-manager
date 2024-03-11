all:
	cargo build


lint check:
	cargo clippy

fix:
	cargo clippy --fix --allow-dirty 

fmt:
	cargo +nightly fmt
