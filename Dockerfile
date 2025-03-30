# Stage 1: Builder - compile the application
FROM rust:1.82-alpine AS builder

# Install build dependencies
RUN apk add --no-cache musl-dev pkgconfig openssl-dev

# Create a non-root user for better security
RUN adduser -D -u 10001 oxicloud

# Create a new empty project and copy only dependency files first
WORKDIR /app
COPY Cargo.toml Cargo.lock ./

# Create empty source files to trick cargo into caching dependencies
RUN mkdir -p src && \
    echo "fn main() {}" > src/main.rs && \
    touch src/lib.rs

# Build dependencies only (this will be cached if dependencies don't change)
RUN cargo build --release

# Remove the fake source files
RUN rm -rf src

# Copy the actual source code
COPY src ./src
COPY db ./db

# Build the actual application
RUN cargo build --release && \
    strip target/release/oxicloud

# Stage 2: Runtime - only include what's necessary for running
FROM alpine:3.21.3

# Install runtime dependencies only
RUN apk add --no-cache libgcc openssl ca-certificates tzdata && \
    rm -rf /var/cache/apk/*

# Create a non-root user
RUN adduser -D -u 10001 oxicloud

# Create app directories with proper permissions
WORKDIR /app
RUN mkdir -p /app/static /app/storage && \
    chown -R oxicloud:oxicloud /app

# Copy only the compiled binary from the builder stage
COPY --from=builder /app/target/release/oxicloud /app/oxicloud

# Copy static files and necessary runtime config
COPY static ./static
COPY db ./db

# Set permissions
RUN chown -R oxicloud:oxicloud /app

# Set the user to run the application
USER oxicloud

# Expose the port the application runs on
EXPOSE 3000

# Run the binary
CMD ["./oxicloud", "--release"]