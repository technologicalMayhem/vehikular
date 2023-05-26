# Using the `rust-musl-builder` as base image, instead of 
# the official Rust toolchain
FROM clux/muslrust:stable AS chef
USER root
RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --bin web-app --recipe-path recipe.json

# Debug
FROM chef AS builder-debug
COPY --from=planner /app/recipe.json recipe.json
# Notice that we are specifying the --target flag!
RUN cargo chef cook -p web-app --target x86_64-unknown-linux-musl --recipe-path recipe.json
COPY . .
RUN cargo build -p web-app --target x86_64-unknown-linux-musl --bin web-app

FROM alpine AS debug
RUN addgroup -S vehikular && adduser -S vehikular -G vehikular
COPY --from=builder-debug /app/target/x86_64-unknown-linux-musl/debug/web-app /usr/local/bin/
USER vehikular
ENV ROCKET_ADDRESS=0.0.0.0
CMD ["/usr/local/bin/web-app"]

# Release
FROM chef AS builder-release
COPY --from=planner /app/recipe.json recipe.json
# Notice that we are specifying the --target flag!
RUN cargo chef cook --release -p web-app --target x86_64-unknown-linux-musl --recipe-path recipe.json
COPY . .
RUN cargo build --release -p web-app --target x86_64-unknown-linux-musl --bin web-app

FROM alpine AS release
RUN addgroup -S vehikular && adduser -S vehikular -G vehikular
COPY --from=builder-release /app/target/x86_64-unknown-linux-musl/release/web-app /usr/local/bin/
USER vehikular
ENV ROCKET_ADDRESS=0.0.0.0
CMD ["/usr/local/bin/web-app"]