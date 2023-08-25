FROM docker.io/library/ubuntu:20.04

# metadata
ARG VCS_REF
ARG BUILD_DATE
ARG IMAGE_NAME

LABEL io.parity.image.authors="devops-team@parity.io" \
	io.parity.image.vendor="Parity Technologies" \
	io.parity.image.title="${IMAGE_NAME}" \
	io.parity.image.description="peer: a platform for web3" \
	io.parity.image.source="https://github.com/paritytech/peer/blob/${VCS_REF}/scripts/ci/dockerfiles/peer_injected_debug.Dockerfile" \
	io.parity.image.revision="${VCS_REF}" \
	io.parity.image.created="${BUILD_DATE}" \
	io.parity.image.documentation="https://github.com/paritytech/peer/"

# show backtraces
ENV RUST_BACKTRACE 1

# install tools and dependencies
RUN apt-get update && \
	DEBIAN_FRONTEND=noninteractive apt-get install -y \
		libssl1.1 \
		ca-certificates && \
# apt cleanup
	apt-get autoremove -y && \
	apt-get clean && \
	find /var/lib/apt/lists/ -type f -not -name lock -delete; \
# add user and link ~/.local/share/peer to /data
	useradd -m -u 1000 -U -s /bin/sh -d /peer peer && \
	mkdir -p /data /peer/.local/share && \
	chown -R peer:peer /data && \
	ln -s /data /peer/.local/share/peer

# add peer binary to docker image
COPY ./peer /usr/local/bin

USER peer

# check if executable works in this container
RUN /usr/local/bin/peer --version

EXPOSE 30333 9933 9944
VOLUME ["/peer"]

ENTRYPOINT ["/usr/local/bin/peer"]
