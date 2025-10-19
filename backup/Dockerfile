FROM debian:bookworm-slim

ENV DEBIAN_FRONTEND=noninteractive

RUN mkdir -p /backups
WORKDIR /backups

RUN apt-get update && apt-get install -y \
    postgresql-client \
    awscli \
    curl \
    wget \
    gnupg \
    gnupg1 \
    gnupg2 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN sh -c 'echo "deb http://apt.postgresql.org/pub/repos/apt $(lsb_release -cs)-pgdg main" > /etc/apt/sources.list.d/pgdg.list'
RUN wget --quiet -O - https://www.postgresql.org/media/keys/ACCC4CF8.asc | apt-key add -

RUN apt-get update && apt-get install -y \
    postgresql-client

COPY backup/backup.sh /backups
RUN chmod +x /backups/backup.sh

ENTRYPOINT ["sleep", "infinity"]