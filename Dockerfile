FROM rust:1.68 as builder
WORKDIR /usr/src/myapp
COPY . .
RUN cargo install --path .

FROM debian:bullseye-slim
LABEL org.opencontainers.image.authors="Ian McLinden"
COPY --from=builder /usr/local/cargo/bin/jiconfi /usr/local/bin/jiconfi
RUN mkdir -p /config
WORKDIR /config

EXPOSE 9000
ENTRYPOINT [ "/usr/local/bin/jiconfi" ]