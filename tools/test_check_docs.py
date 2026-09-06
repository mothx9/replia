#!/usr/bin/env python3
"""Adversarial documentation fixtures; never edit the working documentation."""
from pathlib import Path
import tempfile
import unittest

import check_docs as checks


class DocumentationGuard(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="replai-doc-check-")
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.paths = checks.inventory(checks.ROOT)
        for path in self.paths:
            target = self.root / path
            target.parent.mkdir(parents=True, exist_ok=True)
            if path.endswith((".md", ".json")):
                target.write_bytes((checks.ROOT / path).read_bytes())
            else:
                target.touch()

    def append(self, path, text):
        with (self.root / path).open("a") as stream:
            stream.write("\n" + text + "\n")
        self.paths.add(path)

    def reject(self, expected):
        errors, _ = checks.check_local(self.root, self.paths)
        self.assertTrue(any(expected in error for error in errors), errors)

    def test_current_documents_are_admitted(self):
        errors, diagrams = checks.check_local(self.root, self.paths)
        self.assertEqual(errors, [])
        self.assertTrue(diagrams)

    def test_missing_relative_link(self):
        self.append("README.md", "[missing](docs/missing.md)")
        self.reject("README.md: missing link target: docs/missing.md")

    def test_missing_anchor(self):
        self.append("README.md", "[missing](docs/interaction.md#absent)")
        self.reject("README.md: missing anchor: docs/interaction.md#absent")

    def test_orphan_and_unreferenced_asset(self):
        self.append("docs/orphan.md", "# Orphan")
        self.append("assets/orphan.svg", "<svg/>")
        self.reject("docs/orphan.md: unreachable")
        self.reject("assets/orphan.svg: unreachable")

    def test_retired_reference_and_reintroduced_surface(self):
        self.append("README.md", "[old](docs/baseline.md)")
        self.reject("README.md: missing link target: docs/baseline.md")
        self.append("docs/baseline.md", "# Old")
        self.reject("docs/baseline.md: retired/archive surface")

    def test_archive_and_duplicate_status_owner(self):
        (self.root / "docs/archive").mkdir()
        self.append("docs/archive/report.md", "# Project status")
        self.reject("docs/archive/report.md: retired/archive surface")
        self.reject("docs/archive/report.md: project-status owner")

    def test_abi_version_and_tags(self):
        target = self.root / "docs/c-api.md"
        target.write_text(target.read_text().replace("ABI 1", "ABI 9").replace(
            "| CAPACITY | 4 |", "| CAPACITY | 99 |"))
        self.reject("docs/c-api.md: ABI identity differs")
        self.reject("docs/c-api.md: REPLAI_CAPACITY must have one table value 4")

    def test_link_cannot_escape_repository(self):
        self.append("README.md", "[outside](../../outside.md)")
        self.reject("README.md: link escapes repository")

    def test_reference_html_fences_and_duplicate_heading_anchors(self):
        self.append("README.md", '''
## Repeat
## Repeat
[first](#repeat) [second](#repeat-1)
[guide][interaction]
[interaction]: docs/interaction.md
<a href="docs/presentation.md#external-output">Output</a>
```text
[not a link](missing-example.md)
```
`[also not a link](missing.md)`
''')
        errors, _ = checks.check_local(self.root, self.paths)
        self.assertEqual(errors, [])

    def test_unclosed_fence(self):
        self.append("README.md", "```mermaid\nflowchart LR\n A --> B")
        self.reject("README.md: line")
        errors, _ = checks.check_local(self.root, self.paths)
        self.assertTrue(any("unclosed code fence" in error for error in errors))

    def test_actual_mermaid_parser_rejects_invalid_syntax(self):
        errors = checks.check_mermaid([
            {"file": "docs/bad.md", "line": 12, "code": "flowchart LR\n A --> ["},
        ])
        self.assertTrue(any("docs/bad.md:12: invalid Mermaid" in error for error in errors), errors)


if __name__ == "__main__":
    unittest.main(verbosity=2)
