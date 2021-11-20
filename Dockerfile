FROM rust:buster as builder
WORKDIR /builder
COPY . .
RUN apt update
RUN apt install -y make protobuf-compiler
RUN rustup component add rustfmt
RUN make compile CFLAGS=--release

FROM alpine/git as wodule-setup
WORKDIR /wodules
RUN git clone https://github.com/sagacious-labs/hyperion-network-watcher.git --depth=1

FROM zlim/bcc
WORKDIR /hyperion
COPY --from=builder /builder/target/release/ .
COPY --from=wodule-setup /wodules/hyperion-network-watcher /usr/share/hyperion/wodule/hyperion-network-watcher
CMD [ "./hyperion" ]