.PHONY: default all all_release serve serve_release build build_release test clean

PROJECT := airtifex-web

default:
	@echo "Command list:"
	@cat Makefile | head -n 1 | tr ' ' '\n' | grep -v .PHONY

all: clean test serve

all_release: clean test serve_release

serve:
	@trunk serve --release

serve_release:
	@trunk serve --release


build:
	@trunk build

build_release:
	@trunk build --release

test:
	cargo t

clean:
	@rm -rf target/*
