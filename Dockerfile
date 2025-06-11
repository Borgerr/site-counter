FROM rust:1.87

WORKDIR /usr/src/site_counter
COPY . .

RUN cargo install --path .

ENV START_URL = "wikipedia.com"

CMD ["site_counter"]

