FROM node:alpine AS ui-builder
WORKDIR /usr/src/reflective/ui

COPY ui/package.json ui/package-lock.json ./

RUN npm install
COPY ./ui/ ./
RUN npm run build

FROM rust:1.81 AS chef 
RUN cargo install cargo-chef 
WORKDIR /usr/src/reflective/service

FROM chef AS planner
COPY ./service .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /usr/src/reflective/service/recipe.json recipe.json

RUN cargo chef cook --release --recipe-path recipe.json

COPY ./service .
COPY --from=ui-builder /usr/src/reflective/ui/dist ./ui
RUN cargo build --release --bin reflective 

FROM debian:bookworm-slim AS runtime

WORKDIR /usr/src/app/
COPY --from=builder /usr/src/reflective/service/target/release/reflective /usr/src/app/
ENTRYPOINT ["/usr/src/app/reflective"]