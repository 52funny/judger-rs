.PHONY: all output
TARGET_DIR := ./target
BIN_NAME := judger-rs

debug ?=

ifdef debug
	release :=
	TARGET_DIR := $(TARGET_DIR)/debug
else
	release := --release
	TARGET_DIR := $(TARGET_DIR)/release
endif

all: build run

build:
	$(RUST_FLAGS) cargo build $(release)

run:
	$(TARGET_DIR)/$(BIN_NAME)

clean:
	cargo clean

output:
	mkdir -p ./output
	cp $(TARGET_DIR)/$(BIN_NAME) ./output/$(BIN_NAME)

cp:
	cp $(TARGET_DIR)/$(BIN_NAME) ./demo/$(BIN_NAME)
