# Stage 1: Plan the build
FROM rust:alpine AS planner
WORKDIR /app
# --> FIX: Update apk cache before adding packages in a single layer
RUN apk update && apk add --no-cache \
    build-base \
    pkgconfig \
    openssl-dev \
    openssl-libs-static
RUN cargo install cargo-chef
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Stage 2: Cook the dependencies
FROM rust:alpine AS cachera
WORKDIR /app
# --> FIX: Update apk cache before adding packages in a single layer
RUN apk update && apk add --no-cache \
    build-base \
    pkgconfig \
    openssl-dev \
    openssl-libs-static
RUN cargo install cargo-chef
COPY --from=planner /app/recipe.json recipe.json
COPY Cargo.toml Cargo.lock ./
COPY libs libs
# RUN cargo chef cook --release --recipe-path recipe.json
RUN cargo chef cook --recipe-path recipe.json

# Stage 3: Build the application
FROM rust:alpine AS builder
WORKDIR /app
# --> FIX: Update apk cache before adding packages in a single layer
RUN apk update && apk add --no-cache \
    build-base \
    pkgconfig \
    openssl-dev \
    openssl-libs-static
COPY . .
# Copy over the cached dependencies
# ---> FIX: Changed 'cacher' to 'cachera' to match the stage name
COPY --from=cachera /app/target target
COPY --from=cachera /usr/local/cargo /usr/local/cargo
# RUN cargo build --release
RUN cargo build

# Stage 4: Create the final, small image
FROM alpine:latest
RUN apk add --no-cache openssl ca-certificates
RUN addgroup -S appgroup && adduser -S appuser -G appgroup
# COPY --from=builder /app/target/release/library-service /usr/local/bin/library-service
COPY --from=builder /app/target/debug/library-service /usr/local/bin/library-service
USER appuser
CMD ["library-service"]