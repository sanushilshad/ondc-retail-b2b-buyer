# [...]
# Runtime stage
FROM debian:bullseye-slim AS runtime
WORKDIR /app
# Install OpenSSL - it is dynamically linked by some of our dependencies
# Install ca-certificates - it is needed to verify TLS certificates
# when establishing HTTPS connections
RUN apt-get update -y \
&& apt-get install -y --no-install-recommends openssl ca-certificates \
# Clean up
&& apt-get autoremove -y \
&& apt-get clean -y \
121
&& rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/rust_test rust_test
COPY configuration configuration
ENV APP_ENVIRONMENT production
ENTRYPOINT ["./rust_test"]
