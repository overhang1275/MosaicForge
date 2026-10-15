"""Independent QA for local engine PDFs; requires pypdf, Pillow, numpy and Poppler."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

import numpy as np
from PIL import Image, ImageFilter
from pypdf import PdfReader
from pypdf.generic import ArrayObject, DictionaryObject, IndirectObject, ContentStream


def inspect(path: Path, expected_pages: int, bleed_last: float) -> dict:
    reader = PdfReader(path, strict=True)
    assert len(reader.pages) == expected_pages
    seen = set()
    images = 0
    forms = 0

    def walk(value):
        nonlocal images, forms
        if isinstance(value, IndirectObject):
            key = (value.idnum, value.generation)
            if key in seen:
                return
            seen.add(key)
            value = value.get_object()
        if isinstance(value, DictionaryObject):
            assert not any(k in value for k in ("/JavaScript", "/JS", "/Launch", "/OpenAction", "/AA", "/URI", "/EmbeddedFiles"))
            images += value.get("/Subtype") == "/Image"
            forms += value.get("/Subtype") == "/Form"
            for child in value.values():
                walk(child)
        elif isinstance(value, (ArrayObject, list)):
            for child in value:
                walk(child)

    walk(reader.trailer.raw_get("/Root"))
    actual = {(key, generation) for generation, entries in reader.xref.items()
              for key in entries if key != 0 and generation != 65535}
    assert actual == seen, ("Unreachable or missing PDF objects", actual ^ seen)
    assert images == 0, "Vector export unexpectedly contains images"
    coordinates = 0
    operators = 0

    def content(stream, resources, width, height, visited):
        nonlocal coordinates, operators
        parsed = ContentStream(stream, reader)
        used_forms, used_states = set(), set()
        for args, op in parsed.operations:
            operators += 1
            if op in (b"m", b"l"):
                x, y = map(float, args)
                assert -0.0001 <= x <= width + 0.0001
                assert -0.0001 <= y <= height + 0.0001
                coordinates += 1
            elif op == b"Do":
                name = args[0]
                used_forms.add(name)
                ref = resources["/XObject"].raw_get(name)
                if ref.idnum not in visited:
                    visited.add(ref.idnum)
                    obj = ref.get_object()
                    content(obj, obj["/Resources"], width, height, visited)
            elif op == b"gs":
                used_states.add(args[0])
        assert set(resources.get("/XObject", {})) == used_forms
        assert set(resources.get("/ExtGState", {})) == used_states

    for index, page in enumerate(reader.pages):
        bleed = bleed_last if index == expected_pages - 1 else 0
        expected = (100 + 2 * bleed) * 72 / 25.4
        width, height = float(page.mediabox.width), float(page.mediabox.height)
        assert abs(width - expected) < 0.001 and abs(height - expected) < 0.001
        assert list(page.cropbox) == list(page.mediabox) == list(page.bleedbox)
        assert abs(float(page.trimbox.left) - bleed * 72 / 25.4) < 0.001
        assert abs(float(page.trimbox.width) - 100 * 72 / 25.4) < 0.001
        content(page.get_contents(), page["/Resources"], width, height, set())
    return {"pages": expected_pages, "bytes": path.stat().st_size,
            "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
            "objects": len(seen), "form_xobjects": forms,
            "image_xobjects": images, "operators": operators,
            "bounded_path_points": coordinates}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("directory", type=Path)
    parser.add_argument("--pdf", default="mosaic-100mm.pdf")
    parser.add_argument("--pages", type=int, required=True)
    parser.add_argument("--bleed-last", type=float, default=0)
    parser.add_argument("--pdftoppm", required=True)
    args = parser.parse_args()
    directory = args.directory.resolve()
    pdf = directory / args.pdf
    result = inspect(pdf, args.pages, args.bleed_last)
    subprocess.run([args.pdftoppm, "-r", "300", "-png", str(pdf), str(directory / "qa-page")], check=True)
    metrics = []
    for index in range(args.pages):
        a = np.array(Image.open(directory / f"qa-page-{index+1}.png").convert("RGB"))
        b = np.array(Image.open(directory / f"reference-{index}.png").convert("RGB"))
        assert a.shape == b.shape
        diff = np.abs(a.astype(float) - b.astype(float))
        ink_a, ink_b = a.min(2) < 245, b.min(2) < 245
        grow = lambda mask: np.array(Image.fromarray(mask.astype("uint8") * 255).filter(ImageFilter.MaxFilter(5))) > 0
        extra = ink_a & ~grow(ink_b)
        missing = ink_b & ~grow(ink_a)
        # Poppler and resvg treat the final fractional pixel at the physical page edge differently.
        interior = (slice(1, -1), slice(1, -1))
        row = {"index": index, "mae": float(diff.mean()), "rmse": float(np.sqrt((diff * diff).mean())),
               "max_channel_error": float(diff.max()),
               "extra_ink_far_interior": int(extra[interior].sum()),
               "missing_ink_far_interior": int(missing[interior].sum()),
               "missing_ink_far_including_edge": int(missing.sum())}
        # Regression limits derived from measured fixtures at 300 DPI, not CPU/GPU parity claims.
        assert row["mae"] <= 1.3 and row["rmse"] <= 6.0, row
        assert row["extra_ink_far_interior"] == row["missing_ink_far_interior"] == 0, row
        metrics.append(row)
    result["visual_300dpi"] = metrics
    metadata = directory / "result.json"
    if metadata.exists():
        wire = json.loads(metadata.read_text())
        assert wire["documentHash"] == result["sha256"]
        assert wire["pageCount"] == result["pages"]
        records = json.loads((directory / "pages.json").read_text())
        accepted = [r for r in records if r["variant"]["status"] == "accepted"]
        assert [r["pageNumber"] for r in accepted] == list(range(1, args.pages + 1))
        for record in accepted:
            index = record["variant"]["index"]
            data = (directory / f"source-{index}.svg").read_bytes()
            assert hashlib.sha256(data).hexdigest() == record["variant"]["artifactHash"]
    (directory / "qa-summary.json").write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
