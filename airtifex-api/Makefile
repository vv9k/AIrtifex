.PHONY: default all all_release serve serve_release serve_pg serve_release_pg clean build build_release build_pg build_release_pg

PROJECT := airtifex-api

default:
	@echo "Command list:"
	@cat Makefile | head -n 1 | tr ' ' '\n' | grep -v .PHONY

all: clean test serve

all_release: clean test serve_release

serve:
	@cargo run -- serve

serve_release:
	@cargo run --release -- serve

serve_pg:
	@cargo run --no-default-features --features postgres -- serve

serve_release_pg:
	@cargo run --no-default-features --features postgres --release -- serve

flamegraph:
	@cargo-flamegraph flamegraph

build:
	@cargo build

build_release:
	@cargo build --release

build_pg:
	@cargo build --no-default-features --features postgres

build_release_pg:
	@cargo build --no-default-features --features postgres --release

test:
	cargo t

clean:
	@rm -rf target/*
