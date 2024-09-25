# Define reusable variables
AWS_PROFILE = localstack

# Bring up/down docker containers like localstack, db, redis, ...
up-docker-compose:
	docker-compose -f docker-compose.yml up -d
	sleep 20
down-docker-compose:
	docker-compose -f docker-compose.yml down -v

sqlx-migrate:
	sqlx migrate run --database-url=postgres://local:local@localhost:5432/local --source src/db/sql

sqlx-revert:
	sqlx migrate revert --database-url=postgres://local:local@localhost:5432/local --source src/db/sql --target-version 0

aws:
	export AWS_PROFILE=$(AWS_PROFILE)
	awslocal s3 mb s3://local --region us-west-2 || true
	awslocal sqs create-queue --queue-name local --region us-west-2 || true

up-infra: up-docker-compose sqlx-migrate aws
down-infra: down-docker-compose

# Generate the JWT keys as base64 format
gen-jwt-key:
	@ssh-keygen -t rsa -b 4096 -m PEM -E SHA512 -f jwtRS512.key -N ""
	@openssl rsa -in jwtRS512.key -pubout -outform PEM -out jwtRS512.key.pub
	@echo "\nJWT_PRIVATE_KEY_BASE64\n$$(cat jwtRS512.key | base64)"
	@echo "\nJWT_PUBLIC_KEY_BASE64\n$$(cat jwtRS512.key.pub | base64)"
	@rm -f jwtRS512.key
	@rm -f jwtRS512.key.pub

# Format the code using cargo fmt
fmt:
	cargo fmt --all
fmt-check:
	cargo fmt --all --check

# Install the binaries
install:
	cargo install --path .

# Build the binaries
build:
	cargo build --release

# Test the application
test:
	export AWS_PROFILE=$(AWS_PROFILE); cargo test

# Targets to start the API and worker
start-api:
	export AWS_PROFILE=$(AWS_PROFILE); cargo run api
start-worker:
	export AWS_PROFILE=$(AWS_PROFILE); cargo run worker

build-docker:
	docker build -t rust-service -f src/.deploy/dockerfiles/Dockerfile .
