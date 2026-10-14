"""Prepare only the supplied test.svg export for local engine checks; not a product importer."""
from pathlib import Path
import hashlib
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "test.svg"
OUTPUT = ROOT / "out" / "test-prepared.svg"
NS = "http://www.w3.org/2000/svg"
CSS = ".cls-1{fill:none;stroke:#00a5f1;stroke-width:0.09;stroke-miterlimit:10}"
PAINT = {"fill": "none", "stroke": "#00a5f1", "stroke-width": "0.09", "stroke-miterlimit": "10"}


def prepare(data: bytes) -> tuple[bytes, int]:
    if b"<!DOCTYPE" in data or b"<!ENTITY" in data:
        raise ValueError("DTD/entities are not supported")
    root = ET.fromstring(data)
    styles = list(root.iter(f"{{{NS}}}style"))
    if len(styles) != 1 or "".join((styles[0].text or "").split()) != CSS:
        raise ValueError("Expected the single known cls-1 stylesheet; inspect other exports separately")
    original_paths = [e.attrib["d"] for e in root.iter(f"{{{NS}}}path")]
    converted = 0
    for parent in root.iter():
        for child in list(parent):
            if child.tag in {f"{{{NS}}}style", f"{{{NS}}}title"}:
                parent.remove(child)
        parent.attrib.pop("data-name", None)
        if "class" in parent.attrib:
            if parent.attrib.pop("class") != "cls-1":
                raise ValueError("Unexpected CSS class")
            # CSS has precedence over presentation attributes in this particular export.
            parent.attrib.update(PAINT)
            converted += 1
    assert [e.attrib["d"] for e in root.iter(f"{{{NS}}}path")] == original_paths
    ET.register_namespace("", NS)
    return ET.tostring(root, encoding="utf-8", xml_declaration=True), converted


if __name__ == "__main__":
    original = SOURCE.read_bytes()
    prepared, count = prepare(original)
    # Self-check the only CSS conversion supported by this fixture preparation.
    tiny = f'<svg xmlns="{NS}"><defs><style>{CSS}</style></defs><path class="cls-1" d="M0 0L1 1"/></svg>'.encode()
    tiny_out, tiny_count = prepare(tiny)
    assert tiny_count == 1 and b'stroke="#00a5f1"' in tiny_out and b'class=' not in tiny_out
    OUTPUT.parent.mkdir(exist_ok=True)
    if OUTPUT.exists():
        if OUTPUT.read_bytes() != prepared:
            raise FileExistsError("Prepared output already exists with different bytes")
    else:
        with OUTPUT.open("xb") as handle:
            handle.write(prepared)
    assert SOURCE.read_bytes() == original
    print(f"Source SHA256={hashlib.sha256(original).hexdigest()}; unchanged")
    print(f"Prepared {count} class attributes; {len(prepared)} bytes -> {OUTPUT}")
