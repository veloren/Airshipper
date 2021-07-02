# Airshipper Server Runtime Environment.
FROM debian:buster-slim as runtime

RUN apt-get update
RUN export DEBIAN_FRONTEND=noninteractive
RUN apt-get install -y --no-install-recommends --assume-yes \
        ca-certificates \
        build-essential \
        libssl-dev

WORKDIR /opt/app

COPY --from=builder /app/target/release/server .
COPY ./airshipper-server /opt/app/airshipper-server

ENV RUST_BACKTRACE=full
EXPOSE 8000
CMD [ "./airshipper-server" ]