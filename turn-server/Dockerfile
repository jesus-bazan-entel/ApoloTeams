FROM ghcr.io/mycrl/turn-server:latest

COPY config.toml /etc/turn-server/config.toml

EXPOSE 3478/udp
EXPOSE 3478/tcp

CMD ["--config", "/etc/turn-server/config.toml"]
