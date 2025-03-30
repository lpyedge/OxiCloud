FROM alpine:3.21.3 AS builder

COPY . /Oxicloud

WORKDIR /Oxicloud

RUN apk update && \
    apk add cargo pkgconfig openssl-dev

RUN cargo build --release

# /Oxicloud/target/release/oxicloud

FROM alpine:3.21.3

COPY . /Oxicloud

COPY --from=builder /Oxicloud/target/release/ /Oxicloud
COPY --from=builder /usr/lib/libgcc_s.so.1 /usr/lib/libgcc_s.so.1

WORKDIR /Oxicloud

CMD ["./oxicloud","--release"]