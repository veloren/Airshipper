FROM archlinux/base

RUN pacman -Sy rustup base-devel --noconfirm
RUN rustup set profile minimal
RUN rustup default nightly

COPY . /app

WORKDIR /app/

RUN cargo build --bin server --release
CMD cargo run --bin server --release
