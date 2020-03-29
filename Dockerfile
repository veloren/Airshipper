# Based on https://alexbrand.dev/post/how-to-package-rust-applications-into-minimal-docker-containers/
# and https://benjamincongdon.me/blog/2019/12/04/Fast-Rust-Docker-Builds-with-cargo-vendor/
FROM archlinux/base AS build

# Install deps
RUN pacman -Sy base-devel musl rustup --noconfirm

# Download the target for static linking.
RUN rustup set profile minimal
RUN rustup default nightly
RUN rustup target add x86_64-unknown-linux-musl --toolchain nightly

# Create a dummy project and vendor the app's dependencies.
RUN USER=root cargo new airshipper
RUN cd airshipper && USER=root cargo new server --lib
RUN cd airshipper && USER=root cargo new client --lib --name airshipper_client
WORKDIR airshipper
COPY Cargo.toml Cargo.lock ./
COPY client/Cargo.toml ./client/
COPY server/Cargo.toml ./server/

RUN mkdir .cargo
RUN cargo vendor > .cargo/config

# Copy the source and build the application.
COPY src ./src
COPY server/src ./server/src
COPY server/migrations ./server/migrations
RUN cargo build --release --no-default-features --features server --target x86_64-unknown-linux-musl

# Copy the statically-linked binary into a scratch container.
FROM alpine
WORKDIR /opt/app
COPY --from=build airshipper/target/x86_64-unknown-linux-musl/release/airshipper .
EXPOSE 8000
CMD ["./airshipper"]