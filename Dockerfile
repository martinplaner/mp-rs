FROM rust:alpine AS build-env

WORKDIR /build/

RUN apk add --no-cache musl-dev

COPY . .

RUN cargo build --release


FROM alpine

WORKDIR /app/

COPY --from=build-env /build/target/release/mp-rs /app/

USER nobody
ENTRYPOINT [ "/app/mp-rs", "--file", "/data/words.txt" ]
