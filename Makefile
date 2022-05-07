.PHONY: docs open-docs test

docs:
	cargo doc --all-features

open-docs:
	cargo doc --all-features --open

test:
	cargo test
	cargo test --package tux --all-features
	cargo test --package tux --no-default-features
	cargo test --package tux --no-default-features --features diff
	cargo test --package tux --no-default-features --features exec
	cargo test --package tux --no-default-features --features server
	cargo test --package tux --no-default-features --features temp
	cargo test --package tux --no-default-features --features testdata
	cargo test --package tux --no-default-features --features text
