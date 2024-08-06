ARG RUST_VSN='1.78'

##### Build
FROM cgr.dev/chainguard/wolfi-base as builder
ARG RUST_VSN

RUN apk -U upgrade && apk add --no-cache \
        build-base \
        openssl-dev \
        pax-utils \
        rust-$RUST_VSN

WORKDIR /fpush
COPY / ./
RUN cargo build --release

RUN mkdir -p /rootfs/etc/fpush \
 && mv $(find target/ -name fpush -type f -executable) /rootfs/fpush \
 && touch /rootfs/etc/fpush/settings.json \
 && scanelf --needed --nobanner --format '%n#p' --recursive "$PWD" \
    | tr ',' '\n' \
    | sort -u \
    | awk 'system("[ -e $PWD" $1 " ]") == 0 { next } { print "so:" $1 }' \
        > /rootfs/runDeps

##### Runtime
FROM cgr.dev/chainguard/wolfi-base AS release
COPY --from=builder /rootfs /

RUN apk -U upgrade && apk add --no-cache \
        $(cat /runDeps) \
        tini \
 && apk del --repositories-file /dev/null \
        apk-tools \
        busybox \
        wolfi-base \
        wolfi-keys \
        zlib

ENV RUST_LOG="info"

ENTRYPOINT ["/sbin/tini","--"]
CMD ["/fpush","/etc/fpush/settings.json"]
