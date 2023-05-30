FROM rust:alpine3.17

COPY target/release/greenie-auth-module .env.dev .env.production /

ENTRYPOINT ["/greenie-auth-module"]