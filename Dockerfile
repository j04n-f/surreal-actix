FROM clux/muslrust:1.85.0-stable as builder

WORKDIR /volume

COPY . .

RUN cargo build --release --target x86_64-unknown-linux-musl

FROM gcr.io/distroless/cc-debian12

COPY --from=builder /volume/target/x86_64-unknown-linux-musl/release/surreal-actix ./surreal-actix

ENTRYPOINT [ "/surreal-actix" ]
