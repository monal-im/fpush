# Dockerfile for fpush

This folder holds an example Dockerfile.

To build the image, run the following command from the root of this repository:

```bash
docker build -t localhost/fpush:latest -f docker/Dockerfile .
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
