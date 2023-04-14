.PHONY: default all all_release serve serve_release docker_build docker_build_release create_db clean build build_release

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

flamegraph:
	@cargo-flamegraph flamegraph
build:
	@cargo build

build_release:
	@cargo build --release

docker_build:
	@docker build -t $(PROJECT):latest -f Dockerfile ..

docker_build_release:
	@docker build -t $(PROJECT):latest --build-arg mode=release -f Dockerfile ..

#create_db:
#	@mkdir db
#	@touch db/test.db

test:
	cargo t

clean:
#	@rm -rf db
	@rm -rf target/*
