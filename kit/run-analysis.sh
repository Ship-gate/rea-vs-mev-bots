#!/usr/bin/env bash
set -euo pipefail

research_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
mkdir -p "${research_dir}/output"

docker --context colima-rea-research run --rm \
  --platform linux/amd64 \
  --network none \
  --read-only \
  --tmpfs /tmp:rw,nosuid,nodev,size=128m \
  --cap-drop ALL \
  --security-opt no-new-privileges=true \
  --pids-limit 128 \
  --memory 1g \
  --cpus 2 \
  --mount "type=bind,source=${research_dir}/input,target=/input,readonly" \
  rea-research:6.7.0 inspect-evm-interface /input/runtime.hex hex --json \
  > "${research_dir}/output/interface.json"

printf 'Saved REA output to %s/output/interface.json\n' "${research_dir}"
