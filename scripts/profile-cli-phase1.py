"""Small reproducible Windows release CLI baseline; no installation or file replacement."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys


def save(path, value):
    with path.open("x", encoding="utf-8") as handle:
        json.dump(value, handle, indent=2)


def main():
    root = Path(__file__).resolve().parents[1]
    output = root / "out/phase1-6.6"
    output.mkdir()
    original = root / "test.svg"
    original_hash = hashlib.sha256(original.read_bytes()).hexdigest()
    spec = json.loads((root / "out/cli-6.3-svg.spec.json").read_text(encoding="utf-8-sig"))
    spec["count"] = 3
    save(output / "spec-svg.json", spec)
    for format_name in ["svg", "png"]:
        workload = dict(spec, count=12, output=dict(spec["output"], format=format_name))
        save(output / f"workload-{format_name}.json", workload)
    pdf_job = json.loads((root / "out/pdf-5.6-release/job.json").read_text())
    save(output / "workload-pdf.json", pdf_job)
    exe = root / "services/engine/target/release/mosaicforge.exe"
    cases = [(format_name, n, 1) for format_name in ["svg", "png"] for n in [1, 7, 100]]
    cases += [("png", 7, 2), ("pdf", 7, 1)]
    summaries, baselines = [], {}
    for format_name, n, threads in cases:
        label = f"{format_name}-n{n}-t{threads}"
        destination = output / label
        metrics = output / f"{label}.json"
        arguments = ["generate-pdf" if format_name == "pdf" else "generate",
                     "--template", str(root / "out/test-prepared.svg"),
                     "--spec", str(output / f"workload-{format_name}.json"),
                     "--out", str(destination), "--batch-size", str(n)]
        if format_name != "pdf":
            arguments += ["--threads", str(threads)]
        subprocess.run([sys.executable, str(root / "scripts/measure-test-memory.py"),
                        "--exe", str(exe), "--cwd", str(root), "--output", str(metrics),
                        "--scope", "End-to-end release CLI: input, generation, validation, export, hashing, publication and manifest; excludes compiler/monitor",
                        "--", *arguments], check=True, stdout=subprocess.DEVNULL)
        rows = [json.loads(line) for line in (destination / "manifest.jsonl").read_text().splitlines()]
        assert rows[-1]["status"] == "completed"
        hashes = {}
        if format_name == "pdf":
            doc = next(row["result"] for row in rows if row["kind"] == "pdf_document")
            data = (destination / "mosaic.pdf").read_bytes()
            assert data == (root / "out/pdf-5.6-release/mosaic-100mm.pdf").read_bytes()
            assert hashlib.sha256(data).hexdigest() == doc["documentHash"]
            assert doc["pageCount"] == 3
            hashes["mosaic.pdf"] = doc["documentHash"]
        else:
            results = [row["result"] for row in rows if row["kind"] == "variant"]
            assert [int(row["index"]) for row in results] == list(range(12))
            for row in results:
                if row["status"] == "accepted":
                    name = f"variant-{row['index']}.{format_name}"
                    digest = hashlib.sha256((destination / name).read_bytes()).hexdigest()
                    assert digest == row["artifactHash"]
                    hashes[name] = digest
                    if int(row["index"]) < 3:
                        assert (destination / name).read_bytes() == (root / f"out/cli-6.3-{format_name}" / name).read_bytes()
            assert rows[-1]["accepted"] + rows[-1]["rejected"] == 12
            if format_name in baselines:
                assert hashes == baselines[format_name]
            baselines[format_name] = hashes
        assert {path.name for path in destination.iterdir()} == {"manifest.jsonl", *hashes}
        measured = json.loads(metrics.read_text())
        row = dict(label=label, format=format_name, count=3 if format_name == "pdf" else 12,
                   batchSize=n, threads=threads, accepted=rows[-1]["accepted"],
                   rejected=rows[-1]["rejected"], seconds=measured["duration_seconds"],
                   peakWorkingSetMiB=measured["peak_working_set_bytes"] / 1048576,
                   peakPrivateCommitMiB=measured["peak_private_commit_bytes"] / 1048576,
                   samples=measured["samples"], artifactBytes=sum((destination / name).stat().st_size for name in hashes))
        summaries.append(row)
        print(json.dumps(row), flush=True)
    assert hashlib.sha256(original.read_bytes()).hexdigest() == original_hash
    save(output / "cli-summary.json", dict(cases=summaries, originalTemplateSha256=original_hash,
         scope="One measured run per case; 50ms monitor polling. Workloads differ between SVG/PNG and PDF. No statistical speedup or whole-RAM bound claim."))


if __name__ == "__main__":
    main()
