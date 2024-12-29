# Dockerfile for fpush

This folder holds an example Dockerfile.

To build the image, run the following command from the root of this repository:

```bash
docker buildx build -t localhost/fpush:latest -f docker/Dockerfile .
```

Run the image with:

```bash
docker run --init -d \
    --name fpush \
    -v /path/to/settings.json:/etc/fpush/settings.json \
    -v /path/to/apple.p12:/path/to/apple.p12 \
    -v /path/to/google.json:/path/to/google.json \
    -e RUST_LOG=info \
    localhost/fpush:latest
```

Note: Apple's p12 and/or Google's json file need to be mounted into the
container as you can see in the example above.

Additionally, Apple's p12 file may comes with outdated cipher suites 
which are only support by the OpenSSL legacy provider from version `3.3.3`
on. To make such a p12 file compatibile, you can use the following commands:

```shell
$ openssl pkcs12 -legacy -in apple-old.p12 -nodes -out p12-decrypted.tmp
(enter passphrases if prompted)
$ openssl pkcs12 -in p12-decrypted.tmp -export -out apple-new.p12
(enter passphrases if prompted)
$ rm p12-decrypted.tmp
```
