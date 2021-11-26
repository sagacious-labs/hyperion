FROM rust:buster as builder
WORKDIR /builder
RUN apt update
RUN apt install -y make protobuf-compiler
RUN rustup component add rustfmt
COPY . .
RUN make compile CFLAGS=--release

FROM golang:buster as wodule-setup
WORKDIR /wodules
RUN git clone https://github.com/sagacious-labs/hyperion-network-watcher.git --depth=1
RUN cd hyperion-network-watcher && CGO_ENABLED=0 make compile

FROM archlinux
RUN pacman -Sy --noconfirm bcc bcc-tools python-bcc
WORKDIR /hyperion
COPY --from=builder /builder/target/release/ .
COPY --from=wodule-setup /wodules/hyperion-network-watcher/bin/hyperion-network-watcher /usr/share/hyperion/wodule/
COPY --from=wodule-setup /wodules/hyperion-network-watcher/ebpf/* /usr/share/hyperion/tools/tcp/
CMD [ "./hyperion" ]