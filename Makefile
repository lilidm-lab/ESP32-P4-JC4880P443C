APP   := ESP32-P4-JC4880P443C
TARGET := riscv32imafc-esp-espidf
REL   := target/$(TARGET)/release
NAME  ?= touch_paint

.PHONY: run check clippy example clean

run: example

example:
	cargo run --release --example $(NAME)

check:
	cargo check 2>&1 | tail -3

clippy:
	cargo clippy --all-targets 2>&1 | tail -3

clean:
	cargo clean
