ARG mode=debug
ARG api_addr=127.0.0.1
ARG api_port=6901
ARG database_url=sqlite:///db/test.db
ARG rust_log=info,sqlx=error

FROM rust as builder-release
ENV BUILD_CMD "cargo build --release --bin airtifex-api"

FROM rust as builder-debug
ENV BUILD_CMD "cargo build --bin airtifex-api"

FROM builder-${mode} as builder
WORKDIR app
COPY . .
RUN echo "#/bin/sh\n$BUILD_CMD" > build.sh && chmod +x build.sh && ./build.sh

FROM rust as runtime
WORKDIR app
RUN mkdir -p /app/bin
COPY --from=builder /app/target/${BUILD_MODE:-debug}/airtifex-api /app/bin
ENV API_ADDR $api_addr
ENV API_PORT $api_port
ENV DATABASE_URL $database_url
ENV RUST_LOG $rust_log
EXPOSE $API_PORT
VOLUME ["/db"]
RUN touch /db/test.db
ENTRYPOINT ["/app/bin/airtifex-api"]
