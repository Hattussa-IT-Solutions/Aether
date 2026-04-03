# Aether Programming Language
# Usage:
#   docker build -t aether .
#   docker run --rm aether --version
#   docker run --rm -v $(pwd):/work aether run /work/hello.ae
#   docker run --rm -it aether repl

FROM rust:1.78-slim AS builder
WORKDIR /build
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /build/target/release/aether /usr/local/bin/aether
COPY --from=builder /build/examples /examples

ENTRYPOINT ["aether"]
CMD ["--version"]
