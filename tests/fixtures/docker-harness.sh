#!/bin/sh
set -eu

if [ -n "${RRC_DOCKER_LOG:-}" ]; then
  printf '%s\n' "$*" >> "$RRC_DOCKER_LOG"
fi

case " $* " in
  *" config --format json "*)
    command cat "$RRC_COMPOSE_CONFIG"
    ;;
  *" ps --quiet "*)
    printf '%s\n' 'rrc-sample-container'
    ;;
  *" --command "*)
    printf '%s\n' '1'
    ;;
  *)
    if [ "${1:-}" = "inspect" ]; then
      printf '%s\n' 'healthy'
    fi
    ;;
esac
