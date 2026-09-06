# password-generator -- multi-stage build: static musl binary -> minimal runtime.
#
# The runtime stage stays bare Alpine: the binary is self-contained.

# --- 1. build ----------------------------------------------------------------
FROM rust:1-alpine AS build
RUN apk add --no-cache musl-dev
WORKDIR /src

# Dependency layer: compile all crates against a dummy main so this layer is
# cached until Cargo.toml/Cargo.lock change.
COPY Cargo.toml Cargo.lock ./
RUN mkdir src \
    && echo 'fn main() {}' > src/main.rs \
    && cargo build --release --locked \
    && rm -r src

COPY src ./src
# touch: make cargo notice the real sources are newer than the dummy build.
RUN touch src/main.rs && cargo build --release --locked

# --- 2. runtime --------------------------------------------------------------
FROM alpine:3.21
RUN adduser -S -D -H password-generator
COPY --from=build /src/target/release/password-generator /usr/local/bin/password-generator

USER password-generator
EXPOSE 3000

# --host 0.0.0.0 is required -- the built-in default binds 127.0.0.1, which
# is unreachable from outside the container.
ENTRYPOINT ["/usr/local/bin/password-generator"]
CMD ["serve", "--host", "0.0.0.0", "--port", "3000"]
