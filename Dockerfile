# Debian stable and slim don't work, throwing a glibc error
FROM ubuntu

WORKDIR /app

COPY ./target/release/zero2prod .
COPY ./configuration.yaml .

ENTRYPOINT ["./zero2prod"]
EXPOSE 8080