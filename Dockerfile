FROM rust:1.81-slim as builder

WORKDIR /usr/src/

COPY . .

RUN apt update -y && apt install nodejs npm dav1d pkg-config git python3 python3-pip ninja-build pkg-config meson nasm -y
ENV DAV1D_DIR=dav1d_dir
ENV LIB_PATH=lib/x86_64-linux-gnu
ENV HOME=/usr/src
RUN git clone --branch 1.3.0 --depth 1 https://code.videolan.org/videolan/dav1d.git \
    && cd dav1d \
    && meson build -Dprefix=$HOME/$DAV1D_DIR -Denable_tools=false -Denable_examples=false --buildtype release \
    && ninja -C build \
    && ninja -C build install
ENV PKG_CONFIG_PATH=/usr/src/$DAV1D_DIR/$LIB_PATH/pkgconfig
ENV LD_LIBRARY_PATH=/usr/src/$DAV1D_DIR/$LIB_PATH
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt update -y && apt install curl wget htop vim -y

WORKDIR /usr/app

COPY --from=builder /usr/src/config /usr/app/config
COPY --from=builder /usr/src/dav1d_dir/lib/x86_64-linux-gnu/* /usr/lib/x86_64-linux-gnu/
COPY --from=builder /usr/src/target/release/greedy /usr/app/greedy

ENTRYPOINT ["/usr/app/greedy"]
