# Debian stable and slim don't work, throwing a glibc error
FROM ubuntu

WORKDIR /app

ENV ENVIRONMENT production

COPY ./target/release/zero2prod .
COPY ./configuration ./configuration

ENTRYPOINT ["./zero2prod"]
EXPOSE 8080