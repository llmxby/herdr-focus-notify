#!/bin/sh
set -eu

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
JAR="$SCRIPT_DIR/target/dx-notify-helper-0.1.0-jar-with-dependencies.jar"

if [ ! -f "$JAR" ]; then
  "$SCRIPT_DIR/build.sh" >/dev/null
fi

exec java -jar "$JAR"

