"""Deterministic, dependency-free publication artifact rendering."""

from __future__ import annotations

import html
import json
from pathlib import Path
from typing import Any


def _number(value: Any) -> str:
    return "unavailable" if value is None else f"{float(value):.6g}"


def figure_svg(comparisons: dict[str, Any], output: Path) -> None:
    """Render a vector overview of the first available comparison metric per cell."""

    points: list[tuple[str, float]] = []
    for cell, metrics in sorted(comparisons.items()):
        if not metrics:
            continue
        metric, data = next(iter(sorted(metrics.items())))
        points.append((f"{cell} {metric}", float(data["mean_difference_vz_minus_baseline"])))
    height = max(180, 80 + 36 * len(points))
    scale = max([1.0, *(abs(value) for _, value in points)])
    lines = [
        f'<svg xmlns="http://www.w3.org/2000/svg" width="1100" height="{height}" viewBox="0 0 1100 {height}">',
        '<style>text{font-family:ui-monospace,monospace;font-size:14px}.axis{stroke:#48525c}.positive{fill:#1f7a52}.negative{fill:#b94747}</style>',
        f'<line class="axis" x1="550" y1="40" x2="550" y2="{height - 30}"/>',
        '<text x="30" y="24">Vinglish Zero minus baseline mean difference (first available metric per cell)</text>',
    ]
    for index, (label, value) in enumerate(points):
        y = 55 + index * 36
        width = abs(value) / scale * 460
        x = 550 if value >= 0 else 550 - width
        style = "positive" if value >= 0 else "negative"
        lines.extend((
            f'<text x="20" y="{y + 14}">{html.escape(label)}</text>',
            f'<rect class="{style}" x="{x:.2f}" y="{y}" width="{width:.2f}" height="20"/>',
            f'<text x="{560 if value >= 0 else max(20, x - 100):.2f}" y="{y + 14}">{value:.6g}</text>',
        ))
    lines.append("</svg>")
    output.write_text("\n".join(lines) + "\n", encoding="utf-8")


def distribution_svg(summary: dict[str, Any], output: Path) -> None:
    """Render a vector range-and-mean overview from recorded arm distributions."""

    points: list[tuple[str, dict[str, Any], dict[str, Any]]] = []
    for cell, metrics in sorted(summary.items()):
        metric_names = sorted(
            key.removeprefix("baseline/")
            for key in metrics
            if key.startswith("baseline/") and f"vinglish_zero/{key.removeprefix('baseline/')}" in metrics
        )
        if metric_names:
            metric = metric_names[0]
            points.append((f"{cell} {metric}", metrics[f"baseline/{metric}"], metrics[f"vinglish_zero/{metric}"]))
    height = max(180, 80 + 42 * len(points))
    largest = max([1.0, *(max(float(data["maximum"]) for data in pair) for _, *pair in points)])
    lines = [
        f'<svg xmlns="http://www.w3.org/2000/svg" width="1100" height="{height}" viewBox="0 0 1100 {height}">',
        '<style>text{font-family:ui-monospace,monospace;font-size:14px}.base{stroke:#48525c}.baseline{stroke:#445f9c}.vz{stroke:#1f7a52}</style>',
        '<text x="30" y="24">Observed range and mean (first shared metric per cell)</text>',
    ]
    for index, (label, baseline, vz) in enumerate(points):
        y = 55 + index * 42
        lines.append(f'<text x="20" y="{y + 14}">{html.escape(label)}</text>')
        for offset, data, style in ((0, baseline, "baseline"), (14, vz, "vz")):
            scale = lambda value: 500 + float(value) / largest * 520
            start, end, mean = scale(data["minimum"]), scale(data["maximum"]), scale(data["mean"])
            lines.extend((
                f'<line class="{style}" x1="{start:.2f}" y1="{y + offset}" x2="{end:.2f}" y2="{y + offset}"/>',
                f'<circle class="{style}" cx="{mean:.2f}" cy="{y + offset}" r="4" fill="currentColor"/>',
            ))
    lines.append("</svg>")
    output.write_text("\n".join(lines) + "\n", encoding="utf-8")


def markdown(summary: dict[str, Any], comparisons: dict[str, Any]) -> str:
    lines = [
        "# Controlled Generation Study Report",
        "",
        "This report is generated solely from recorded observations. It reports association, not causation.",
        "",
        "## Reproducibility",
        "",
        "Every row is linked to a durable observation, command logs, artifact hash, and frozen manifest provenance.",
        "",
        "## Arm Comparisons",
        "",
        "| Benchmark / level | Metric | VZ - baseline mean | 95% bootstrap CI | Randomization p | Cliff's delta |",
        "| --- | --- | ---: | --- | ---: | ---: |",
    ]
    rows = 0
    for cell, metrics in sorted(comparisons.items()):
        for metric, value in sorted(metrics.items()):
            interval = value["mean_difference_ci_95"]
            ci = "unavailable" if interval is None else f"[{_number(interval[0])}, {_number(interval[1])}]"
            lines.append(
                f"| {cell} | {metric} | {_number(value['mean_difference_vz_minus_baseline'])} | {ci} | "
                f"{_number(value['randomization_p_value_two_sided'])} | {_number(value['cliffs_delta_vz_minus_baseline'])} |"
            )
            rows += 1
    if not rows:
        lines.extend(("| No paired observations available | - | - | - | - | - |", ""))
    lines.extend((
        "",
        "## Descriptive Statistics",
        "",
        "The machine-readable `summary.json` retains all metric-level distributions, confidence intervals, variance, and outlier counts.",
        "",
        "## Interpretation Limits",
        "",
        "Do not aggregate heterogeneous domains into a universal quality score. Multiple comparisons remain exploratory unless a correction was frozen before data collection.",
    ))
    return "\n".join(lines) + "\n"


def html_report(markdown_report: str, figure_name: str) -> str:
    body = "\n".join(f"<p>{html.escape(line)}</p>" if line else "" for line in markdown_report.splitlines())
    return f"""<!doctype html>
<html lang=\"en\"><head><meta charset=\"utf-8\"><title>Controlled Generation Study</title>
<style>body{{max-width:1100px;margin:48px auto;font:16px/1.55 system-ui,sans-serif;color:#18212b}} pre{{white-space:pre-wrap}} img{{max-width:100%}}</style>
</head><body><h1>Controlled Generation Study</h1><img src=\"{html.escape(figure_name)}\" alt=\"Comparison overview\"><pre>{html.escape(markdown_report)}</pre></body></html>
"""


def _pdf_escape(value: str) -> str:
    return value.replace("\\", "\\\\").replace("(", "\\(").replace(")", "\\)")


def minimal_pdf(lines: list[str]) -> bytes:
    """Emit a standards-compliant multipage PDF without an unpinned renderer dependency."""

    pages = [lines[index : index + 52] for index in range(0, max(1, len(lines)), 52)] or [[]]
    font_id = 3 + 2 * len(pages)
    page_ids = [3 + 2 * index for index in range(len(pages))]
    objects = [
        b"<< /Type /Catalog /Pages 2 0 R >>",
        f"<< /Type /Pages /Kids [{' '.join(f'{page} 0 R' for page in page_ids)}] /Count {len(pages)} >>".encode(),
    ]
    for index, page in enumerate(pages):
        page_id = page_ids[index]
        content_id = page_id + 1
        content = ["BT", "/F1 10 Tf", "50 760 Td"]
        for line_index, line in enumerate(page):
            if line_index:
                content.append("0 -13 Td")
            content.append(f"({_pdf_escape(line[:140])}) Tj")
        content.append("ET")
        stream = "\n".join(content).encode("latin-1", errors="replace")
        objects.extend((
            f"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Resources << /Font << /F1 {font_id} 0 R >> >> /Contents {content_id} 0 R >>".encode(),
            b"<< /Length " + str(len(stream)).encode() + b" >>\nstream\n" + stream + b"\nendstream",
        ))
    objects.append(b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>")
    data = bytearray(b"%PDF-1.4\n")
    offsets = [0]
    for index, obj in enumerate(objects, 1):
        offsets.append(len(data))
        data.extend(f"{index} 0 obj\n".encode())
        data.extend(obj)
        data.extend(b"\nendobj\n")
    xref = len(data)
    data.extend(f"xref\n0 {len(objects) + 1}\n0000000000 65535 f \n".encode())
    for offset in offsets[1:]:
        data.extend(f"{offset:010d} 00000 n \n".encode())
    data.extend(f"trailer\n<< /Size {len(objects) + 1} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n".encode())
    return bytes(data)


def render(summary_path: Path, comparisons_path: Path, output: Path) -> None:
    """Generate Markdown, HTML, SVG, and PDF reports from computed statistics only."""

    summary = json.loads(summary_path.read_text(encoding="utf-8"))
    comparisons = json.loads(comparisons_path.read_text(encoding="utf-8"))
    output.mkdir(parents=True, exist_ok=True)
    document = markdown(summary, comparisons)
    figure_svg(comparisons, output / "comparison-overview.svg")
    distribution_svg(summary, output / "distribution-overview.svg")
    (output / "publication-report.md").write_text(document, encoding="utf-8")
    (output / "publication-report.html").write_text(html_report(document, "comparison-overview.svg"), encoding="utf-8")
    (output / "publication-report.pdf").write_bytes(minimal_pdf(document.splitlines()))
