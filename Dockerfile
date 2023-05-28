FROM rust:alpine3.17 as builder

ENV RUSTFLAGS="-C target-feature=-crt-static"
RUN apk add --no-cache musl-dev

WORKDIR /app
COPY ./ /app

RUN cargo build --release
RUN strip target/release/greenie-auth-module

# use a plain alpine image, the alpine version needs to match the builder
FROM alpine:3.17
RUN apk add --no-cache libgcc

COPY --from=builder /app/target/release/greenie-auth-module .
COPY --from=builder /app/.env.dev .
COPY --from=builder /app/.env.production .

ENV APP_ENV=dev
ENTRYPOINT ["/greenie-auth-module"]