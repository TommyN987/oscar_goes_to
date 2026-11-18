# One image holds the binary and the built SPA (spec rule 2.4.1).

FROM node:26.13.1-trixie-slim AS spa
WORKDIR /build/frontend
RUN npm install --global pnpm@12.21.0
COPY frontend/package.json frontend/pnpm-lock.yaml frontend/pnpm-workspace.yaml ./
RUN pnpm install --frozen-lockfile
COPY frontend/ ./
RUN pnpm run build

FROM rust:1.99.1-slim-trixie AS binary
WORKDIR /build
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates/ crates/
RUN cargo build --release --locked --bin gold-ballot

FROM gcr.io/distroless/cc-debian13:nonroot
COPY --from=binary /build/target/release/gold-ballot /app/gold-ballot
COPY --from=spa /build/frontend/build /app/spa
ENV SPA_DIR=/app/spa
# Cloud Run and Render override PORT; this default serves a plain `docker run`.
ENV PORT=8080
EXPOSE 8080
USER nonroot
ENTRYPOINT ["/app/gold-ballot"]
