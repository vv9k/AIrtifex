.PHONY: default api api_release clean

PROJECT := airtifex
SERVER_ADDR := 127.0.0.1
SERVER_PORT := 6901

default:
	@echo "Command list:"
	@cat Makefile | head -n 1 | tr ' ' '\n' | grep -v .PHONY

api:
	@cd $(PROJECT)-api && $(MAKE) all

api_release:
	@cd $(PROJECT)-api && $(MAKE) all_release

web:
	@cd $(PROJECT)-web && $(MAKE) all

web_release:
	@cd $(PROJECT)-web && $(MAKE) all_release

api_flamegraph:
	@cd $(PROJECT)-api && $(MAKE) flamegraph

build_docker:
	@docker build -t $(PROJECT) .

run_docker: build_docker
	@docker-compose up -d

stop_docker:
	@docker-compose down

lint:
	cargo fmt --check --all
	cargo clippy --all-targets --features postgres --no-default-features -- -Dclippy::all
	cargo clippy --all-targets --features sqlite --no-default-features -- -Dclippy::all

clean:
	@rm -rf target \
 		   $(PROJECT)-api/target \
 		   $(PROJECT)-core/target \
