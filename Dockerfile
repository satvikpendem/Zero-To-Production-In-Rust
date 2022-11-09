# Compile outside Docker and run
# Debian stable and slim don't work, throwing a glibc error
FROM ubuntu:latest
WORKDIR /app

RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl ca-certificates \
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

# ENVIRONMENT equaling `production` is required for sqlx to run offline and migrate properly
ENV ENVIRONMENT production
COPY ./target/release/zero2prod .
# Configuration files are copied over but will be overwritten by environment variables injected from the PaaS or server
COPY ./configuration ./configuration
COPY ./migrations ./migrations
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