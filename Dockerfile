FROM node:alpine AS ui-builder
WORKDIR /usr/src/reflective/ui
COPY ui/package.json ui/package-lock.json ./
RUN npm install
COPY ./ui/ ./
RUN npm run build

FROM rust:trixie AS builder
WORKDIR /usr/src/reflective/service
COPY ./service .
COPY --from=ui-builder /usr/src/reflective/ui/dist ./ui
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/usr/src/reflective/service/target \
    SQLX_OFFLINE=true cargo build --release --bin reflective \
    && cp target/release/reflective /reflective

FROM debian:trixie-slim AS runtime
RUN apt update && apt install -y ca-certificates openssl && rm -rf /var/lib/apt/lists/*
WORKDIR /usr/src/app/
COPY --from=builder /reflective /usr/src/app/reflective

ENTRYPOINT ["/usr/src/app/reflective"]