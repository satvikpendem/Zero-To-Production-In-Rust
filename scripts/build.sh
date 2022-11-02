cargo build --release --target-dir ./target
docker build --tag zero2prod --progress=plain .
