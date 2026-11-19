#!/usr/bin/env python3
"""Compile the real native bundle against a minimal Horizon-shaped fixture."""
import json
from pathlib import Path
import subprocess
import tempfile
import tomllib
import horizon


def run(command, cwd):
    result = subprocess.run(command, cwd=cwd, capture_output=True, text=True)
    if result.returncode:
        raise RuntimeError(result.stdout + result.stderr)
    return result


def main():
    root = horizon.ROOT
    mixins = horizon.MIXINS
    run(["cargo", "build", "--locked", "--offline", "-p", "horizon-mixins-cli"], mixins)
    cli = mixins / "target/debug/horizon-mixins"
    manifest_path = next((root / "mixins").glob("*/horizon-mixin.toml"))
    manifest = tomllib.loads(manifest_path.read_text())
    with tempfile.TemporaryDirectory(prefix="ultimine-fixture-", dir=root / "target") as scratch:
        scratch = Path(scratch)
        original, staged = scratch / "original", scratch / "staged"
        game = original / "crates/horizon-client/src"
        game.mkdir(parents=True)
        (original / "Cargo.toml").write_text('[workspace]\nmembers=["crates/horizon-client"]\nresolver="2"\n')
        (original / "crates/horizon-client/Cargo.toml").write_text('[package]\nname="horizon-client"\nversion="0.1.0"\nedition="2024"\n')
        (game / "lib.rs").write_text("pub mod game;\n")
        (game / "game.rs").write_text("""pub struct Game; impl Game { pub fn break_block(&mut self, pos:[i32;3])->i32 { pos[0] } }\n""")
        (original / "rust-toolchain.toml").write_text((root / "rust-toolchain.toml").read_text())
        (original / ".horizon-revision").write_text(horizon.PIN + "\n")
        run(["cargo", "generate-lockfile", "--offline"], original)
        base = ["--workspace", str(original), "--mods", str(root / "mixins")]
        run([str(cli), "check", *base, "--out", str(staged), "--trust-native-mods"], root)
        test_result = run(["cargo", "test", "--locked", "--offline"], staged)
        assert "test result: ok" in test_result.stdout
        plan = scratch / "plan.json"
        run([str(cli), "plan", *base, "--out", str(plan)], root)
        report = json.loads(plan.read_text())
        assert report["schema"] == horizon.COMPAT["mixin_schema"]
        assert report["injections"][0]["mod_id"] == manifest["mod"]["id"]
    print(json.dumps({"status": "ok", "mixin_fixture_compiled": True, "runtime_tests_passed": True}, indent=2))


if __name__ == "__main__":
    main()
