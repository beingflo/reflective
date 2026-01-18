#!/usr/bin/env bash
die() { echo "$*" 1>&2 ; exit 1; }

trap "rm -f .env.prod" EXIT
export SOPS_AGE_KEY=$(op item get "SOPS age key - reflective" --reveal --fields "private key") || die "Failed to get age key from 1Password"
sops -d --input-type dotenv --output-type dotenv .env.prod.enc > .env.prod || die "Failed to decrypt .env.prod file"

docker --context arm compose --file compose.prod.yml pull || die "Failed to pull new image"
docker --context arm compose --file compose.prod.yml up --build -d || die "Failed to bring compose up"