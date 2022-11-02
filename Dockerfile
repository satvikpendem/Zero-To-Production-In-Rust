# Compile outside Docker and run
# Debian stable and slim don't work, throwing a glibc error
FROM ubuntu
WORKDIR /app
ENV ENVIRONMENT production
COPY ./target/release/zero2prod .
COPY ./configuration ./configuration
ENTRYPOINT ["./zero2prod"]
EXPOSE 8080

# # Compile inside Docker and run
# FROM rust:latest
# WORKDIR /app
# RUN apt update && apt install lld clang -y
# COPY . .
# ENV SQLX_OFFLINE true
# RUN cargo build --release
# ENV ENVIRONMENT production
# ENTRYPOINT ["./target/release/zero2prod"]