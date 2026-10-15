"""Measure one Windows test process using OS peak counters, without compiling during measurement."""
import argparse
import ctypes
from ctypes import wintypes
import json
import os
from pathlib import Path
import subprocess
import time


class Counters(ctypes.Structure):
    _fields_ = [("cb", wintypes.DWORD), ("PageFaultCount", wintypes.DWORD)] + [
        (name, ctypes.c_size_t) for name in (
            "PeakWorkingSetSize", "WorkingSetSize", "QuotaPeakPagedPoolUsage",
            "QuotaPagedPoolUsage", "QuotaPeakNonPagedPoolUsage", "QuotaNonPagedPoolUsage",
            "PagefileUsage", "PeakPagefileUsage", "PrivateUsage")]


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--exe", type=Path, required=True)
    parser.add_argument("--cwd", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("arguments", nargs=argparse.REMAINDER)
    args = parser.parse_args()
    assert os.name == "nt", "This monitor requires Windows PSAPI"
    arguments = args.arguments[1:] if args.arguments[:1] == ["--"] else args.arguments
    query = ctypes.WinDLL("psapi", use_last_error=True).GetProcessMemoryInfo
    query.argtypes = [wintypes.HANDLE, ctypes.POINTER(Counters), wintypes.DWORD]
    query.restype = wintypes.BOOL
    peak_working_set = peak_private_commit = samples = 0
    started = time.perf_counter()
    log = args.output.with_suffix(".log")
    with log.open("xb") as handle:
        process = subprocess.Popen([str(args.exe.resolve()), *arguments], cwd=args.cwd, stdout=handle, stderr=subprocess.STDOUT)
        while True:
            counters = Counters()
            counters.cb = ctypes.sizeof(counters)
            if not query(wintypes.HANDLE(int(process._handle)), ctypes.byref(counters), counters.cb):
                raise ctypes.WinError(ctypes.get_last_error())
            peak_working_set = max(peak_working_set, counters.PeakWorkingSetSize)
            peak_private_commit = max(peak_private_commit, counters.PeakPagefileUsage)
            samples += 1
            if process.poll() is not None:
                break
            time.sleep(0.05)
    result = {"exit_code": process.returncode, "duration_seconds": time.perf_counter() - started,
              "peak_working_set_bytes": peak_working_set,
              "peak_private_commit_bytes": peak_private_commit, "samples": samples,
              "command": [str(args.exe.resolve()), *arguments],
              "scope": "Entire test process including template generation, PDF export and PNG references; excludes compiler and parent monitor"}
    with args.output.open("x") as handle:
        json.dump(result, handle, indent=2)
    print(json.dumps(result, indent=2))
    assert process.returncode == 0, log


if __name__ == "__main__":
    main()
