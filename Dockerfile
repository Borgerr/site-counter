FROM rust:1.87

WORKDIR /usr/src/site_counter
COPY . .

RUN cargo install --path .

ENV START_URL="https://wikipedia.com"
ENV DST_FILE="./archive.xz"

CMD ["site_counter", "$START_URL",  "-d", "$DST_FILE"]

