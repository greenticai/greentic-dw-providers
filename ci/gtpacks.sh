#!/usr/bin/env bash
set -euo pipefail

command="${1:-help}"
manifest="${GTPACK_MANIFEST:-packs/gtpacks.manifest.json}"
dist_root="${GTPACK_DIST:-dist}"

usage() {
  cat <<'EOF'
Usage: ci/gtpacks.sh <validate|build|publish>

Commands:
  validate  Check the manifest and, when gtc is available, validate each launcher AnswerDocument against gtc wizard --schema.
  build     Validate and generate all gtpack artifacts with gtc wizard --answers.
  publish   Push generated .gtpack artifacts from dist/ to GHCR via oras.
EOF
}

validate_manifest() {
  python3 - "$manifest" <<'PY'
import json
import pathlib
import sys

manifest_path = pathlib.Path(sys.argv[1])
if not manifest_path.is_file():
    raise SystemExit(f"missing pack manifest: {manifest_path}")

doc = json.loads(manifest_path.read_text(encoding="utf-8"))
if doc.get("version") != 1:
    raise SystemExit("pack manifest version must be 1")

packs = doc.get("packs")
if not isinstance(packs, list) or not packs:
    raise SystemExit("pack manifest must contain a non-empty packs array")

seen_dirs = set()
for index, pack in enumerate(packs):
    if not isinstance(pack, dict):
        raise SystemExit(f"pack entry {index} is not an object")

    category = pack.get("category")
    name = pack.get("name")
    pack_id = pack.get("pack_id")
    if not category or not name or not pack_id:
        raise SystemExit(f"pack entry {index} must define category, name, and pack_id")
    if "/" in category or "/" in name:
        raise SystemExit(f"pack entry {index} uses an invalid category or name")

    pack_dir = f"packs/dw/{category}/{name}-pack"
    if pack_dir in seen_dirs:
        raise SystemExit(f"duplicate pack dir in manifest: {pack_dir}")
    seen_dirs.add(pack_dir)

    if pack.get("pack_dir") not in (None, pack_dir):
        raise SystemExit(f"pack entry {index} has an unexpected pack_dir override")

print(f"validated {len(packs)} gtpack manifest entries")
PY
}

validate_answers_with_gtc() {
  if ! command -v gtc >/dev/null 2>&1; then
    printf 'gtc is not installed; skipping launcher schema validation\n'
    return 0
  fi

  python3 - "$manifest" <<'PY'
import json
import pathlib
import subprocess
import sys
import tempfile

manifest_path = pathlib.Path(sys.argv[1])
doc = json.loads(manifest_path.read_text(encoding="utf-8"))

for pack in doc["packs"]:
    category = pack["category"]
    name = pack["name"]
    pack_id = pack["pack_id"]
    pack_dir = f"packs/dw/{category}/{name}-pack"

    answers = {
        "wizard_id": "greentic-dev.wizard.launcher.main",
        "schema_id": "greentic-dev.launcher.main",
        "schema_version": "1.0.0",
        "locale": "en",
        "answers": {
            "selected_action": "pack",
            "delegate_answer_document": {
                "wizard_id": "greentic-pack.wizard.run",
                "schema_id": "greentic-pack.wizard.answers",
                "schema_version": "1.0.0",
                "locale": "en",
                "answers": {
                    "pack_dir": pack_dir,
                    "create_pack_scaffold": True,
                    "create_pack_id": pack_id,
                    "run_delegate_flow": False,
                    "run_delegate_component": False,
                    "run_doctor": True,
                    "run_build": True,
                    "sign": False
                },
                "locks": {}
            }
        },
        "locks": {}
    }

    with tempfile.TemporaryDirectory() as temp_dir:
        answer_path = pathlib.Path(temp_dir) / "answers.json"
        answer_path.write_text(json.dumps(answers, indent=2) + "\n", encoding="utf-8")
        subprocess.run(
            ["gtc", "wizard", "--schema", "--answers", str(answer_path)],
            check=True,
            stdout=subprocess.DEVNULL,
        )

print(f"validated {len(doc['packs'])} launcher AnswerDocuments with gtc wizard --schema")
PY
}

build_gtpacks() {
  validate_manifest
  validate_answers_with_gtc

  if ! command -v gtc >/dev/null 2>&1; then
    echo "gtc is required to build gtpack artifacts" >&2
    exit 1
  fi

  mkdir -p "$dist_root"
  python3 - "$manifest" "$dist_root" <<'PY'
import json
import pathlib
import shutil
import subprocess
import sys
import tempfile

manifest_path = pathlib.Path(sys.argv[1])
dist_root = pathlib.Path(sys.argv[2])
doc = json.loads(manifest_path.read_text(encoding="utf-8"))

for pack in doc["packs"]:
    category = pack["category"]
    name = pack["name"]
    pack_id = pack["pack_id"]
    pack_dir = pathlib.Path(f"packs/dw/{category}/{name}-pack")
    gtpack_out = dist_root / f"{pack_dir}.gtpack"
    scaffold_gtpack = pack_dir / "dist" / f"{pack_dir.name}.gtpack"

    answers = {
        "wizard_id": "greentic-dev.wizard.launcher.main",
        "schema_id": "greentic-dev.launcher.main",
        "schema_version": "1.0.0",
        "locale": "en",
        "answers": {
            "selected_action": "pack",
            "delegate_answer_document": {
                "wizard_id": "greentic-pack.wizard.run",
                "schema_id": "greentic-pack.wizard.answers",
                "schema_version": "1.0.0",
                "locale": "en",
                "answers": {
                    "pack_dir": str(pack_dir),
                    "create_pack_scaffold": True,
                    "create_pack_id": pack_id,
                    "run_delegate_flow": False,
                    "run_delegate_component": False,
                    "run_doctor": True,
                    "run_build": True,
                    "sign": False
                },
                "locks": {}
            }
        },
        "locks": {}
    }

    with tempfile.TemporaryDirectory() as temp_dir:
        answer_path = pathlib.Path(temp_dir) / "answers.json"
        answer_path.write_text(json.dumps(answers, indent=2) + "\n", encoding="utf-8")
        subprocess.run(
            ["gtc", "wizard", "--answers", str(answer_path), "--yes", "--non-interactive"],
            check=True,
        )

    if not scaffold_gtpack.is_file():
        raise SystemExit(f"missing scaffold gtpack output: {scaffold_gtpack}")

    gtpack_out.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(scaffold_gtpack, gtpack_out)

    if not gtpack_out.is_file():
        raise SystemExit(f"missing gtpack output: {gtpack_out}")

    print(f"built {gtpack_out}")

    shutil.rmtree(pack_dir, ignore_errors=True)

shutil.rmtree(pathlib.Path(".greentic"), ignore_errors=True)

print(f"built {len(doc['packs'])} gtpack artifacts under {dist_root}")
PY
}

publish_gtpacks() {
  if ! command -v oras >/dev/null 2>&1; then
    echo "oras is required to publish gtpack artifacts" >&2
    exit 1
  fi

  version="$(sed -n 's/^version = "\([^"]*\)"/\1/p' Cargo.toml | head -n1)"
  if [ -z "$version" ]; then
    echo "Unable to determine root package version from Cargo.toml" >&2
    exit 1
  fi

  token="${GHCR_TOKEN:-${GITHUB_TOKEN:-}}"
  if [ -z "$token" ]; then
    echo "Missing GHCR_TOKEN or GITHUB_TOKEN" >&2
    exit 1
  fi

  printf '%s' "$token" | oras login ghcr.io -u "${GITHUB_ACTOR:-github-actions[bot]}" --password-stdin

  shopt -s nullglob
  gtpacks=(
    "$dist_root"/packs/dw/*/*-pack.gtpack
    "$dist_root"/*/*-pack.gtpack
  )
  if [ "${#gtpacks[@]}" -eq 0 ]; then
    echo "No gtpack artifacts found under ${dist_root}/packs/dw/** or ${dist_root}/*/**"
    exit 1
  fi

  for gtpack in "${gtpacks[@]}"; do
    if [[ "$gtpack" == "$dist_root"/packs/dw/* ]]; then
      rel="${gtpack#${dist_root}/}"
    elif [[ "$gtpack" == "$dist_root"/* ]]; then
      rel="packs/dw/${gtpack#${dist_root}/}"
    else
      echo "Unexpected gtpack artifact path: ${gtpack}" >&2
      exit 1
    fi
    repo="ghcr.io/greenticai/${rel%.gtpack}"
    tags=("${version}" "latest")

    for tag in "${tags[@]}"; do
      ref="${repo}:${tag}"
      echo "Publishing ${gtpack} -> ${ref}"
      oras push \
        --artifact-type application/vnd.greentic.gtpack \
        --annotation "org.opencontainers.image.title=$(basename "$gtpack")" \
        --annotation "org.opencontainers.image.version=${version}" \
        "$ref" \
        "${gtpack}:application/octet-stream"
    done
  done
}

case "$command" in
  validate)
    validate_manifest
    validate_answers_with_gtc
    ;;
  build)
    build_gtpacks
    ;;
  publish)
    publish_gtpacks
    ;;
  help|-h|--help)
    usage
    ;;
  *)
    echo "Unknown command: $command" >&2
    usage >&2
    exit 1
    ;;
esac
