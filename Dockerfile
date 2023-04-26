FROM debian:latest as chef
RUN apt-get update && apt-get install -y build-essential curl libssl-dev pkg-config git
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
RUN rustup default nightly
RUN rustup target add wasm32-unknown-unknown
RUN cargo install trunk cargo-chef

FROM chef as planner_api
COPY . /airtifex
WORKDIR /airtifex/airtifex-api
RUN cargo chef prepare --recipe-path recipe.json

FROM chef as planner_web
COPY . /airtifex
WORKDIR /airtifex/airtifex-web
RUN cargo chef prepare --recipe-path recipe.json

FROM chef as builder_web
WORKDIR /airtifex/airtifex-web
COPY --from=planner_web /airtifex/airtifex-web/recipe.json web-recipe.json
RUN cargo chef cook --release --recipe-path web-recipe.json --target=wasm32-unknown-unknown
COPY airtifex-api /airtifex/airtifex-api
COPY airtifex-core /airtifex/airtifex-core
COPY airtifex-web /airtifex/airtifex-web
RUN trunk build --release index.html

FROM chef as builder_api
WORKDIR /airtifex/airtifex-api
COPY --from=planner_api /airtifex/airtifex-api/recipe.json api-recipe.json
RUN cargo chef cook --release --recipe-path api-recipe.json --bin airtifex-api --target-dir ../target
COPY airtifex-api /airtifex/airtifex-api
COPY airtifex-core /airtifex/airtifex-core
COPY airtifex-web /airtifex/airtifex-web
RUN cargo build --release --bin airtifex-api --target-dir ../target

FROM debian:latest
RUN apt-get update && apt-get install -y nginx curl unzip libgomp1
RUN curl -o libtorch.zip -L https://download.pytorch.org/libtorch/cpu/libtorch-cxx11-abi-shared-with-deps-2.0.0%2Bcpu.zip
RUN unzip libtorch.zip
COPY --from=builder_web /airtifex/airtifex-web/dist/ /var/www/html/
COPY --from=builder_api /airtifex/target/release/airtifex-api /usr/local/bin/airtifex-api
COPY assets/nginx-vhost.conf /etc/nginx/sites-available/default
COPY assets/api-basic-config.yaml /etc/airtifex/config.yaml
RUN ln -sf /etc/nginx/sites-available/default /etc/nginx/sites-enabled/default
ENV LIBTORCH /libtorch
ENV LD_LIBRARY_PATH /libtorch/lib:$LD_LIBRARY_PATH

COPY assets/docker-entry.sh /usr/local/bin/docker-entry.sh
RUN chmod +x /usr/local/bin/docker-entry.sh /usr/local/bin/airtifex-api

RUN mkdir /srv/airtifex

ENTRYPOINT ["/usr/local/bin/docker-entry.sh"]
