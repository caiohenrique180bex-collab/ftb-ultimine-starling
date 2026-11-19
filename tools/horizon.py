#!/usr/bin/env python3
"""Export pinned Horizon sources, apply the native bundle and build/test the client."""
import argparse
import contextlib
import fcntl
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tarfile
import tempfile

ROOT = Path(__file__).resolve().parents[1]
COMPAT = json.loads((ROOT / "compatibility.json").read_text())
PIN = COMPAT["horizon_revision"]
MIXINS = Path(os.environ.get("HORIZON_MIXINS", ROOT.parent / "horizon-mixins")).resolve()


def digest(data):
    return hashlib.sha256(data).hexdigest()


@contextlib.contextmanager
def lock(path):
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("a") as handle:
        fcntl.flock(handle, fcntl.LOCK_EX)
        yield


def stage(horizon, stages):
    if not (horizon / ".git").exists():
        raise ValueError("--horizon must be a Git checkout")
    subprocess.run(["git", "-C", str(horizon), "cat-file", "-e", PIN + "^{commit}"], check=True)
    signature = digest(PIN.encode())
    stages.mkdir(parents=True, exist_ok=True)
    out = stages / signature[:20]
    if not out.exists():
        temporary = Path(tempfile.mkdtemp(prefix=".ultimine-stage-", dir=stages))
        try:
            process = subprocess.Popen(["git", "-C", str(horizon), "archive", "--format=tar", PIN],
                                       stdout=subprocess.PIPE)
            with tarfile.open(fileobj=process.stdout, mode="r|") as archive:
                archive.extractall(temporary, filter="data")
            if process.wait():
                raise ValueError("git archive failed")
            for path, expected in COMPAT["source_sha256"].items():
                if digest((temporary / path).read_bytes()) != expected:
                    raise ValueError(f"pinned source hash mismatch: {path}")
            (temporary / ".horizon-revision").write_text(PIN + "\n")
            os.rename(temporary, out)
        except BaseException:
            shutil.rmtree(temporary, ignore_errors=True)
            raise
    return out


def apply(horizon):
    revision = subprocess.check_output(["git", "-C", str(MIXINS), "rev-parse", "HEAD"], text=True).strip()
    if revision != COMPAT["horizon_mixins_revision"]:
        raise ValueError(f"horizon-mixins must be pinned at {COMPAT['horizon_mixins_revision']}")
    subprocess.run(["cargo", "build", "--locked", "--offline", "-p", "horizon-mixins-cli"],
                   cwd=MIXINS, check=True)
    original = stage(horizon, ROOT / "target/mixins/original")
    output = ROOT / "target/mixins/modded"
    plan = ROOT / "target/mixins/plan.json"
    cli = MIXINS / "target/debug/horizon-mixins"
    base = ["--workspace", str(original), "--mods", str(ROOT / "mixins")]
    subprocess.run([str(cli), "plan", *base, "--out", str(plan)], check=True)
    subprocess.run([str(cli), "apply", *base, "--out", str(output)], check=True)
    return output


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action", choices=["stage", "check", "test", "build"])
    parser.add_argument("--horizon", type=Path, default=ROOT.parent / "horizon")
    parser.add_argument("--target-dir", type=Path, default=ROOT / "target/horizon-cargo")
    args = parser.parse_args()
    with lock(ROOT / "target/stage.lock"):
        output = apply(args.horizon.resolve())
        report = {"stage": str(output), "horizon_revision": PIN}
        if args.action != "stage":
            command = ["cargo", args.action, "--locked", "--offline", "-p", "horizon-client",
                       "--bin", "horizon-client", "--no-default-features", "--target-dir",
                       str(args.target_dir.resolve())]
            if args.action == "build":
                command += ["--profile", "fast"]
            if args.action == "test":
                command += ["game::ftb_ultimine::tests", "--", "--nocapture"]
            result = subprocess.run(command, cwd=output, capture_output=True, text=True)
            (ROOT / "target" / f"horizon-{args.action}.log").write_text(result.stdout + result.stderr)
            report["cargo_exit"] = result.returncode
            if result.returncode:
                print(result.stderr[-10000:], file=sys.stderr)
                raise SystemExit(result.returncode)
        print(json.dumps(report, indent=2))


if __name__ == "__main__":
    main()
