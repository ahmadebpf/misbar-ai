# syntax=docker/dockerfile:1

# --- Stage 1: build the Rust backend (stable toolchain, unaffected by ezkl's nightly pin) ---
# Needs Debian trixie (glibc 2.41+): the prebuilt onnxruntime static lib that
# `ort`'s download-binaries feature links in references glibc 2.38+ symbols
# (__isoc23_strtoll etc.) that bookworm's glibc 2.36 doesn't have.
FROM rust:1.96-trixie AS builder
WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY backend/Cargo.toml backend/Cargo.toml
COPY backend/src backend/src
COPY backend/migrations backend/migrations
# `ort` (download-binaries) fetches prebuilt ONNX Runtime and links it at
# `cargo build` time, not at container runtime — this stage needs network.
RUN cargo build --release --bin backend

# --- Stage 2: fetch the ezkl CLI and compile the circuit from model.onnx ---
# Uses ezkl's prebuilt Linux release binary (glibc) — no nightly toolchain
# needed here since we're not building ezkl from source. See docs/ezkl.md.
FROM debian:trixie-slim AS circuit
ARG TARGETARCH
ARG EZKL_VERSION=v23.0.5
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl && rm -rf /var/lib/apt/lists/*
RUN case "$TARGETARCH" in \
      amd64) EZKL_ASSET=ezkl-linux-gnu.tar.gz ;; \
      arm64) EZKL_ASSET=ezkl-linux-aarch64.tar.gz ;; \
      *) echo "unsupported arch: $TARGETARCH" >&2; exit 1 ;; \
    esac && \
    curl -fL "https://github.com/zkonduit/ezkl/releases/download/${EZKL_VERSION}/${EZKL_ASSET}" \
      | tar xz -C /usr/local/bin && chmod +x /usr/local/bin/ezkl

WORKDIR /build
COPY model/model.onnx model/model.onnx
COPY backend/scripts backend/scripts
RUN backend/scripts/zk_setup.sh

# --- Stage 3: runtime image ---
# Verified via `ldd` against the builder output: onnxruntime is fully
# statically linked (only libc/libstdc++/libgcc/libm are needed), so a slim
# base is fine here — same glibc generation (trixie) as the builder either way.
FROM debian:trixie-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=circuit /usr/local/bin/ezkl /usr/local/bin/ezkl
COPY --from=builder /build/target/release/backend /app/backend/backend
COPY model/model.onnx /app/model/model.onnx
COPY --from=circuit /build/backend/circuit /app/backend/circuit

WORKDIR /app/backend
EXPOSE 3000
# misbar.key and the sqlite db are runtime state, not build artifacts —
# default them under data/ so `docker run -v <volume>:/app/backend/data`
# persists them across restarts/redeploys without extra -e flags.
RUN mkdir -p data
ENV MISBAR_KEY_PATH=data/misbar.key
ENV MISBAR_DATABASE_URL=sqlite://data/misbar.db
CMD ["./backend"]
