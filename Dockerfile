FROM rust
RUN mkdir /discord-rbot
COPY ./ /
RUN cargo install diesel_cli --no-default-features --features "postgres"
RUN cargo build
CMD /bin/bash -c "diesel migration run & cargo run"