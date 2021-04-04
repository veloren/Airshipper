# Credit goes to https://www.lpalmieri.com/posts/2020-11-01-zero-to-production-5-how-to-deploy-a-rust-application/#3-8-optimising-our-docker-image

# Check for modified dependencies
FROM lukemathwalker/cargo-chef as planner
WORKDIR app
COPY . .
# Compute a lock-like file for our project
RUN cargo chef prepare  --recipe-path recipe.json

# Cache dependencies
FROM lukemathwalker/cargo-chef as cacher
WORKDIR app
COPY --from=planner /app/recipe.json recipe.json
# Build our project dependencies, not our application! 
RUN cargo chef cook --release --recipe-path recipe.json

# Build Image
FROM rust:slim-buster AS builder

RUN apt-get update
RUN export DEBIAN_FRONTEND=noninteractive
RUN apt-get install -y --no-install-recommends --assume-yes \
        ca-certificates \
        build-essential \
        libssl-dev

WORKDIR app
# Copy over the cached dependencies
COPY --from=cacher /app/target target
COPY --from=cacher /usr/local/cargo /usr/local/cargo
COPY . .
# Build our application, leveraging the cached deps!
RUN cargo build --bin server --release --all-features

# Airshipper Server Runtime Environment.
FROM debian:buster-slim as runtime

RUN apt-get update
RUN export DEBIAN_FRONTEND=noninteractive
RUN apt-get install -y --no-install-recommends --assume-yes \
        ca-certificates \
        build-essential \
        libssl-dev

COPY --from=builder /app/target/release/server .

ENV RUST_BACKTRACE=1
WORKDIR /opt/app
EXPOSE 8000
CMD [ "./server" ]