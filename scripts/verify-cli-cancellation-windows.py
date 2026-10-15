"""Local Windows console-signal check; creates a fresh out/ directory, deletes nothing.

Uses the prepared user SVG. Run after building mosaicforge-cli, never during linking.
The child has its own hidden console; CTRL_C_EVENT cannot reach the user's console.
"""
import ctypes
import argparse
from ctypes import wintypes
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import time


def main():
    if sys.platform != "win32":
        raise SystemExit("Windows only")
    root = Path(__file__).resolve().parents[1]
    parser = argparse.ArgumentParser()
    parser.add_argument("--pdf", action="store_true")
    pdf = parser.parse_args().pdf
    job = root / ("out/cli-6.5-pdf-cancel-signal" if pdf else "out/cli-6.4-cancel-signal")
    job.mkdir()  # Exclusive; refuse to overwrite an earlier run.
    source = "out/pdf-5.6-release/job.json" if pdf else "out/cli-6.3-png.spec.json"
    spec = json.loads((root / source).read_text(encoding="utf-8-sig"))
    generation = spec["generation"] if pdf else spec
    generation["count"] = 10000
    spec_path = job / "spec.json"
    spec_path.write_text(json.dumps(spec), encoding="utf-8")
    destination = job / "artifacts"
    startup = subprocess.STARTUPINFO()
    startup.dwFlags |= subprocess.STARTF_USESHOWWINDOW
    startup.wShowWindow = subprocess.SW_HIDE
    command = [str(root / "services/engine/target/debug/mosaicforge.exe"), "generate-pdf" if pdf else "generate",
               "--template", str(root / "out/test-prepared.svg"), "--spec", str(spec_path),
               "--out", str(destination), "--batch-size", "7"]
    if not pdf:
        command += ["--threads", "2"]
    kernel = ctypes.WinDLL("kernel32", use_last_error=True)
    kernel.SetConsoleCtrlHandler.argtypes = [ctypes.c_void_p, wintypes.BOOL]
    kernel.SetConsoleCtrlHandler.restype = wintypes.BOOL
    # The task host may ignore Ctrl+C. Clear that inheritable flag before spawn.
    if not kernel.SetConsoleCtrlHandler(None, False):
        raise ctypes.WinError(ctypes.get_last_error())
    process = subprocess.Popen(command, stdin=subprocess.DEVNULL, stdout=subprocess.PIPE,
                               stderr=subprocess.PIPE, creationflags=subprocess.CREATE_NEW_CONSOLE,
                               startupinfo=startup)
    kernel.AttachConsole.argtypes = [wintypes.DWORD]
    kernel.AttachConsole.restype = wintypes.BOOL
    kernel.FreeConsole.restype = wintypes.BOOL
    kernel.GenerateConsoleCtrlEvent.argtypes = [wintypes.DWORD, wintypes.DWORD]
    kernel.GenerateConsoleCtrlEvent.restype = wintypes.BOOL
    kernel.GetConsoleProcessList.argtypes = [ctypes.POINTER(wintypes.DWORD), wintypes.DWORD]
    kernel.GetConsoleProcessList.restype = wintypes.DWORD
    attached = False
    try:
        deadline = time.monotonic() + 90
        manifest_path = destination / "manifest.jsonl"
        while True:
            if process.poll() is not None:
                raise AssertionError("child exited before cancellation")
            if manifest_path.exists():
                complete = manifest_path.read_bytes().split(b"\n")[:-1]
                if any(json.loads(line)["kind"] == ("pdf_page" if pdf else "variant") for line in complete):
                    break
            if time.monotonic() >= deadline:
                raise TimeoutError("no completed variant within local check timeout")
            time.sleep(0.02)
        kernel.FreeConsole()  # Detach only this helper, never the parent process.
        if not kernel.AttachConsole(process.pid):
            raise ctypes.WinError(ctypes.get_last_error())
        attached = True
        if not kernel.SetConsoleCtrlHandler(None, True):
            raise ctypes.WinError(ctypes.get_last_error())
        clients = (wintypes.DWORD * 8)()
        count = kernel.GetConsoleProcessList(clients, len(clients))
        assert 0 < count <= len(clients)
        assert set(clients[:count]) == {process.pid, os.getpid()}, "unexpected console clients"
        if not kernel.GenerateConsoleCtrlEvent(0, 0):  # CTRL_C_EVENT, isolated console only.
            raise ctypes.WinError(ctypes.get_last_error())
        stdout, stderr = process.communicate(timeout=30)
    finally:
        if attached:
            kernel.FreeConsole()
        if process.poll() is None:
            process.kill()  # Only this check's own child, on failure/timeout.
            process.communicate()
    (job / "stdout.jsonl").write_bytes(stdout)
    (job / "stderr.json").write_bytes(stderr)
    assert process.returncode == 130, (process.returncode, stderr)
    assert json.loads(stderr)["error"]["code"] == "cancelled"
    manifest = [json.loads(line) for line in manifest_path.read_bytes().splitlines()]
    assert manifest[0]["kind"] == "job"
    assert manifest[1:] == [json.loads(line) for line in stdout.splitlines()]
    variants = ([line["record"]["variant"] for line in manifest if line["kind"] == "pdf_page"] if pdf
                else [line["result"] for line in manifest if line["kind"] == "variant"])
    summary = manifest[-1]
    assert summary["status"] == "cancelled"
    assert 0 < len(variants) < generation["count"]
    assert summary["accepted"] + summary["rejected"] == len(variants)
    indices = [int(result["index"]) for result in variants]
    assert indices == sorted(set(indices))
    expected = {"manifest.jsonl"}
    for result in variants:
        if result["status"] == "accepted" and not pdf:
            name = f"variant-{result['index']}.png"
            expected.add(name)
            assert hashlib.sha256((destination / name).read_bytes()).hexdigest() == result["artifactHash"]
    assert {path.name for path in destination.iterdir()} == expected
    if pdf:
        assert not any(line["kind"] == "pdf_document" for line in manifest)
    evidence = {"signal": "CTRL_C_EVENT", "format": "pdf" if pdf else "png", "exit": process.returncode,
                "accepted": summary["accepted"], "rejected": summary["rejected"],
                "manifestRecords": len(manifest), "unrecordedArtifacts": 0, "temporaries": 0}
    (job / "verification.json").write_text(json.dumps(evidence, indent=2), encoding="utf-8")
    print(json.dumps(evidence))


if __name__ == "__main__":
    main()
