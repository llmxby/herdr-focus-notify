#!/bin/sh
set -eu

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
cd "$SCRIPT_DIR"

mvn -q -DskipTests package
echo "$SCRIPT_DIR/target/dx-notify-helper-0.1.0-jar-with-dependencies.jar"

