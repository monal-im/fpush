### alternative tag is e.g. '1.72.0'
ARG RUST_VSN='stable'

##### Build
FROM docker.io/clux/muslrust:${RUST_VSN} as builder

COPY / ./
RUN cargo build --release

RUN mkdir -p /rootfs/etc/fpush \
 && mv $(find target/ -name fpush -type f -executable) /rootfs/fpush \
 && touch /rootfs/etc/fpush/settings.json

##### Runtime
FROM gcr.io/distroless/static-debian12:nonroot AS prod

COPY --from=builder /rootfs /

ENV RUST_LOG=info

ENTRYPOINT ["/fpush","/etc/fpush/settings.json"]
