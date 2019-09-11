FROM rust
RUN mkdir /discord-rbot
COPY ./ /
RUN cargo build
CMD /bin/bash -c "cargo run"