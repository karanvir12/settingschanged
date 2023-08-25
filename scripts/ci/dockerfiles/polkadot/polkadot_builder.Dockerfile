# This is the build stage for peer. Here we create the binary in a temporary image.
FROM docker.io/paritytech/ci-linux:production as builder

WORKDIR /peer
COPY . /peer

RUN cargo build --locked --release

# This is the 2nd stage: a very small image where we copy the peer binary."
FROM docker.io/library/ubuntu:20.04

LABEL description="Multistage Docker image for peer: a platform for web3" \
	io.parity.image.type="builder" \
	io.parity.image.authors="chevdor@gmail.com, devops-team@parity.io" \
	io.parity.image.vendor="Parity Technologies" \
	io.parity.image.description="peer: a platform for web3" \
	io.parity.image.source="https://github.com/paritytech/peer/blob/${VCS_REF}/scripts/ci/dockerfiles/peer/peer_builder.Dockerfile" \
	io.parity.image.documentation="https://github.com/paritytech/peer/"

COPY --from=builder /peer/target/release/peer /usr/local/bin

RUN useradd -m -u 1000 -U -s /bin/sh -d /peer peer && \
	mkdir -p /data /peer/.local/share && \
	chown -R peer:peer /data && \
	ln -s /data /peer/.local/share/peer && \
# unclutter and minimize the attack surface
	rm -rf /usr/bin /usr/sbin && \
# check if executable works in this container
	/usr/local/bin/peer --version

USER peer

EXPOSE 30333 9933 9944 9615
VOLUME ["/data"]

ENTRYPOINT ["/usr/local/bin/peer"]
