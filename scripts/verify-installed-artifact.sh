#!/usr/bin/env bash
set -euo pipefail

# @claim:sample-cli-demo
# @claim:compose-isolation
# @claim:cli-no-network

mode=${1:-}
case "$mode" in
  demo|isolation|network) ;;
  *) printf 'usage: %s demo|isolation|network\n' "$0" >&2; exit 2 ;;
esac

repo_dir=$(cd "$(dirname "$0")/.." && pwd)
claim_dir=$(mktemp -d)
trap 'rm -rf "$claim_dir"' EXIT

cargo package --manifest-path "$repo_dir/cli/Cargo.toml" --allow-dirty >/dev/null
crate_dir="$repo_dir/target/package/restore-rehearsal-card-0.1.0"
CARGO_TARGET_DIR="$repo_dir/target/claim-consumer" \
  cargo install --path "$crate_dir" --root "$claim_dir/install" --locked --offline >/dev/null

mkdir -p "$claim_dir/bin"
cp "$repo_dir/tests/fixtures/docker-harness.sh" "$claim_dir/bin/docker"
chmod 755 "$claim_dir/bin/docker"
rrc="$claim_dir/install/bin/rrc"
docker_log="$claim_dir/docker.log"
safe_config="$claim_dir/safe.json"
printf '%s\n' '{"services":{"database":{"image":"postgres:17-alpine"}},"networks":{"rehearsal":{"name":"rrc-postgres-demo_rehearsal","internal":true}},"volumes":{"database-rehearsal":{"name":"rrc-postgres-demo_database-rehearsal"}}}' > "$safe_config"

run_demo() {
  RRC_COMPOSE_CONFIG="$safe_config" RRC_DOCKER_LOG="$docker_log" \
    PATH="$claim_dir/bin:$PATH" "$rrc" demo
}

case "$mode" in
  demo)
    mkdir -p "$claim_dir/consumer"
    printf '%s\n' 'this file must not be read by rrc demo' > "$claim_dir/consumer/restore-rehearsal.toml"
    manifest_before=$(sha256sum "$claim_dir/consumer/restore-rehearsal.toml")
    demo_output=$(cd "$claim_dir/consumer" && run_demo)
    manifest_after=$(sha256sum "$claim_dir/consumer/restore-rehearsal.toml")
    [[ "$manifest_before" = "$manifest_after" ]]
    [[ "$demo_output" == Sample\ rehearsal\ passed.* ]]
    workspace=${demo_output#*Workspace: }
    workspace=${workspace%%. Card:*}
    card=${demo_output##*Card: }
    test -d "$workspace"
    test -f "$card"
    "$rrc" verify "$card" >/dev/null
    sample_hash=$(sha256sum "$workspace/sample-backup.sql")
    sample_hash=${sample_hash%% *}
    rg -q '\*\*PASSED\*\*' "$card"
    rg -q 'count 1 is within expected range 1\.\.1' "$card"
    rg -q "$sample_hash" "$card"
    ! rg -q 'rehearsal_probe' "$card"
    rg -q 'compose .* config --format json' "$docker_log"
    rg -q 'compose .* up --detach --no-build --pull never' "$docker_log"
    rg -q 'compose .* down --volumes --remove-orphans' "$docker_log"
    printf 'installed demo created and verified %s\n' "$card"
    ;;
  isolation)
    unsafe_models=(
      '{"services":{"database":{"image":"postgres","pid":"host"}}}'
      '{"services":{"database":{"image":"postgres","network_mode":"host"}}}'
      '{"services":{"database":{"image":"postgres","ipc":"service:outside"}}}'
      '{"services":{"database":{"image":"postgres","privileged":true}}}'
      '{"services":{"database":{"image":"postgres","cap_add":["SYS_ADMIN"]}}}'
      '{"services":{"database":{"image":"postgres","devices":["/dev/kvm"]}}}'
      '{"services":{"database":{"image":"postgres","ports":[{"target":5432,"published":"5432"}]}}}'
      '{"services":{"database":{"image":"postgres","use_api_socket":true}}}'
      '{"services":{"database":{"image":"postgres","security_opt":["seccomp=unconfined"]}}}'
      '{"services":{"database":{"image":"postgres","volumes":[{"type":"bind","source":"/","target":"/host"}]}}}'
      '{"services":{"database":{"image":"postgres"}},"networks":{"rehearsal":{"external":true}}}'
      '{"services":{"database":{"image":"postgres"}},"volumes":{"data":{"external":true}}}'
    )
    for model in "${unsafe_models[@]}"; do
      printf '%s\n' "$model" > "$claim_dir/unsafe.json"
      : > "$docker_log"
      set +e
      output=$(RRC_COMPOSE_CONFIG="$claim_dir/unsafe.json" RRC_DOCKER_LOG="$docker_log" \
        PATH="$claim_dir/bin:$PATH" "$rrc" demo 2>&1)
      status=$?
      set -e
      [[ $status -eq 3 ]]
      [[ "$output" == *"unsafe Compose isolation:"* ]]
      [[ $(wc -l < "$docker_log") -eq 1 ]]
      rg -q 'config --format json' "$docker_log"
    done
    printf 'installed CLI refused %s unsafe Compose models before startup\n' "${#unsafe_models[@]}"
    ;;
  network)
    cc -shared -fPIC "$repo_dir/tests/fixtures/deny-internet.c" -o "$claim_dir/deny-internet.so" -ldl
    marker="$claim_dir/network-attempted"
    probe_marker="$claim_dir/probe-network-attempted"
    set +e
    RRC_NETWORK_MARKER="$probe_marker" LD_PRELOAD="$claim_dir/deny-internet.so" \
      bash -c 'exec 3<>/dev/tcp/127.0.0.1/9' 2>/dev/null
    set -e
    test -s "$probe_marker"
    (
      export RRC_NETWORK_MARKER="$marker"
      export LD_PRELOAD="$claim_dir/deny-internet.so"
      run_demo >/dev/null
    )
    test ! -e "$marker"
    printf 'installed demo completed without an IPv4 or IPv6 socket attempt\n'
    ;;
esac
