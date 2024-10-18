watch:
  cargo-watch -x check -s 'cargo fmt && cargo run'
run:
  cargo run
fmt:
  cargo fmt
lint:
  cargo clippy --all-features -- -D warnings -W clippy::pedantic -W clippy::nursery -W rust-2018-idioms
build: lint
  cargo build --release
test:
  cargo test
