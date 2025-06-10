FROM rust:1.87

WORKDIR /usr/src/site_counter
COPY . .

RUN cargo install --path .

CMD ["site_counter"]

