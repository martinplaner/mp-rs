FROM rust AS build-env

WORKDIR /build/

COPY . .

RUN cargo build --release

FROM debian:bookworm-slim

WORKDIR /app/

COPY --from=build-env /build/target/release/mp-rs /app/

USER nobody
ENTRYPOINT [ "/app/mp-rs", "--file", "/data/words.txt" ]
