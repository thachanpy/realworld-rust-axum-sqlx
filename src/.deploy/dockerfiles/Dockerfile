# Use the official Rust image for building
FROM rust:1.81.0 AS build

WORKDIR /usr/src/app

# Copy only the Cargo files to leverage caching
COPY Cargo.toml Cargo.lock ./

# Build dependencies first
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release

# Now copy the source code and build the actual application
COPY src ./src
RUN cargo install --path .

# Use a smaller base image for the final stage
FROM gcr.io/distroless/cc-debian12

WORKDIR /usr/src/app

# Set the architecture argument (arm64, i.e. aarch64 as default)
# For amd64, i.e. x86_64, you can append a flag when invoking the build `... --build-arg "ARCH=x86_64"`
ARG ARCH=aarch64

# Application files
COPY --from=build /usr/local/cargo/bin/api /usr/local/bin/api
COPY src/resources/ ./src/resources/

CMD ["api"]