
FROM rust:1.79.0
WORKDIR /app
COPY . .
RUN chmod +x env.sh && /bin/bash -c "./env.sh"
RUN echo $PATH
RUN cargo build --release
# RUN chmod -R 755 /app
CMD ["./target/release/rust_test"]