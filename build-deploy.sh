#!/usr/bin/env bash
die() { echo "$*" 1>&2 ; exit 1; }

./build || die "Failed to build docker image"
./install || die "Failed to deploy to server"