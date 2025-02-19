FROM node:20 as node-builder

WORKDIR /app

COPY editor ./editor
COPY display ./display
COPY build_static.sh ./build_static.sh

RUN chmod +x build_static.sh && ./build_static.sh

FROM rust:1.84-alpine3.20 as rust-builder

WORKDIR /app

COPY src ./src
COPY Cargo.lock ./Cargo.lock
COPY Cargo.toml ./Cargo.toml

RUN apk add --no-cache openssl-dev alpine-sdk openssl-libs-static

RUN cargo build --release

FROM alpine:3
WORKDIR /app

COPY --from=node-builder /app/static ./static

COPY --from=rust-builder /app/target/release/rpi-widgetbox ./rpi-widgetbox

EXPOSE 3012

CMD [ "./rpi-widgetbox" ]
