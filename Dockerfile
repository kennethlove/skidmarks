FROM rust:1.83-alpine AS builder
RUN apk add musl-dev
RUN apk add libressl-dev
RUN cargo install --locked dioxus-cli

WORKDIR /usr/src/app
COPY . .
WORKDIR /usr/src/app/web
RUN dx build --release

FROM nginx
COPY --from=builder /usr/src/app/target/dx/web/release/web/public /usr/share/nginx/html

