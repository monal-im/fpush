# based on https://mitchellh.com/writing/nix-with-dockerfiles

# Nix builder
FROM nixos/nix:latest AS builder

# Copy our source and setup our working dir.
COPY . /tmp/build
WORKDIR /tmp/build

RUN nix-channel --update

# Build our Nix environment
RUN nix \
    --extra-experimental-features "nix-command flakes" \
    --option filter-syscalls false \
    build

# Copy the Nix store closure into a directory. The Nix store closure is the
# entire set of Nix store values that we need for our build.
RUN mkdir /tmp/nix-store-closure
RUN mkdir /tmp/app
RUN cp -R $(nix-store -qR result/) /tmp/nix-store-closure \
    && ln -s $(readlink -f result)/ /tmp/app/fpush

# Final image is based on chainguard/static. We copy Nix dependencies
# but they're fully self-contained so we don't need Nix anymore.
FROM cgr.dev/chainguard/static:latest

WORKDIR /app

# Copy /nix/store
COPY --from=builder /tmp/nix-store-closure /nix/store
COPY --from=builder /tmp/app /app

ENV RUST_LOG=info

ENTRYPOINT ["/app/fpush/bin/fpush", "/etc/fpush/settings.json"]
