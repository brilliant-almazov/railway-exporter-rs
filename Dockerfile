FROM rust:1.83-alpine AS builder

RUN apk add --no-cache musl-dev pkgconfig openssl-dev openssl-libs-static

WORKDIR /app
COPY Cargo.toml ./
COPY src ./src
RUN cargo build --release

FROM scratch
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
COPY --from=builder /app/target/release/railway-exporter /railway-exporter
EXPOSE 9333
ENTRYPOINT ["/railway-exporter"]
