FROM rust:alpine3.17

WORKDIR /app
COPY target/x86_64-unknown-linux-musl/release/greenie-auth-module .env.dev .env.production ./
RUN ldd /app/greenie-auth-module

ENTRYPOINT ["/app/greenie-auth-module"]