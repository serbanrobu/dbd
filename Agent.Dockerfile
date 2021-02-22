FROM rust:1.50 as planner
WORKDIR app
RUN cargo install cargo-chef
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM rust:1.50 as cacher
WORKDIR app
RUN cargo install cargo-chef
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

FROM rust:1.50 as builder
WORKDIR app
COPY --from=cacher /app/target target
COPY --from=cacher $CARGO_HOME $CARGO_HOME
COPY . .
RUN cargo build --release --bin dbd-agent

FROM debian:buster-slim as runtime
WORKDIR app
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends \
    mariadb-client postgresql-client \
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/dbd-agent dbd-agent
EXPOSE 8080
ENTRYPOINT ["./dbd-agent"]
