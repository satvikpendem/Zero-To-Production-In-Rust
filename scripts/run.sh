cargo build --release --target-dir ./target
docker build -t=zero2prod .
docker run -p 8080:8080 --rm zero2prod | jaq
