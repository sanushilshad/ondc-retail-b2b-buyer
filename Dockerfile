# We use the latest Rust stable release as base image
FROM rust:1.77.1
# Let's switch our working directory to `app` (equivalent to `cd app`)
# The `app` folder will be created for us by Docker in case it does not
# exist already.
WORKDIR /app
# Install the required system dependencies for our linking configuration
# RUN apt update && apt install lld clang -y
# Copy all files from our working environment to our Docker image
COPY . .
RUN chmod +x env.sh && /bin/bash -c "./env.sh"
# Let's build our binary!
# We'll use the release profile to make it faaaast
RUN echo $PATH
RUN cargo build --release
# When `docker run` is executed, launch the binary!
CMD ["./target/release/rust_test"]