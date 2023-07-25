run:
	rm -rf ./bindings && cargo test && APP_ENV=local cargo run

watch:
	RUST_BACKTRACE=1 APP_ENV=local cargo watch -x run