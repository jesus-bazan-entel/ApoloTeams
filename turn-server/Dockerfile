FROM coturn/coturn:latest

COPY turnserver.conf /etc/turnserver.conf

EXPOSE 3478/udp
EXPOSE 3478/tcp
EXPOSE 49152-49155/udp

CMD ["-c", "/etc/turnserver.conf"]
