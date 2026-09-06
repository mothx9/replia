#!/usr/bin/env python3
"""Read-only documentation structure, ABI prose and Mermaid qualification."""
import argparse
from html import unescape
from html.parser import HTMLParser
import json
from pathlib import Path
import re
import subprocess
from urllib.parse import unquote, urlsplit

ROOT = Path(__file__).resolve().parents[1]
RETIRED = {"docs/baseline.md", "docs/archaeology.md"}
OWNERS = {
    "README.md", "CONTRIBUTING.md", "AGENTS.md", "ROADMAP.md", "CHANGELOG.md",
    "docs/README.md", "docs/repl.md", "docs/architecture.md",
    "docs/interaction.md", "docs/presentation.md", "docs/c-api.md",
    "docs/development.md",
}


def inventory(root):
    """Include concurrent/untracked additions, omit ignored and deleted paths."""
    output = subprocess.check_output(
        ["git", "ls-files", "-z", "--cached", "--others", "--exclude-standard"],
        cwd=root,
    )
    return {p for p in output.decode().split("\0") if p and (root / p).is_file()}


class HtmlLinks(HTMLParser):
    def __init__(self):
        super().__init__()
        self.links = []
        self.ids = set()

    def handle_starttag(self, tag, attrs):
        attrs = dict(attrs)
        for key in ("href", "src"):
            if key in attrs and attrs[key] is not None:
                self.links.append(attrs[key])
        for key in ("id", "name"):
            if attrs.get(key):
                self.ids.add(attrs[key])


def parse_markdown(text):
    """Parse the repository's Markdown links/headings and fenced diagrams.

    Supports inline/reference links, HTML href/src/id, ATX/setext headings,
    duplicate heading slugs and backtick/tilde fences. Not a general GFM renderer.
    """
    visible, diagrams, errors = [], [], []
    fence, info, body, start = None, "", [], 0
    for number, line in enumerate(text.splitlines(), 1):
        marker = re.match(r"^ {0,3}(`{3,}|~{3,})(.*)$", line)
        if fence:
            if marker and marker[1][0] == fence[0] and len(marker[1]) >= len(fence) and not marker[2].strip():
                if info == "mermaid":
                    diagrams.append({"line": start, "code": "\n".join(body)})
                fence = None
            else:
                body.append(line)
            visible.append("")
        elif marker:
            fence, info, body, start = marker[1], marker[2].strip(), [], number
            visible.append("")
        else:
            visible.append(line)
    if fence:
        errors.append(f"line {start}: unclosed code fence")
    prose = "\n".join(visible)
    html = HtmlLinks()
    html.feed(prose)
    ids, seen, headings = html.ids, set(), []
    for i, line in enumerate(visible):
        match = re.match(r"^ {0,3}#{1,6}\s+(.+?)\s*#*\s*$", line)
        title = match[1] if match else None
        if i and re.fullmatch(r" {0,3}(?:=+|-+)\s*", line) and visible[i - 1].strip():
            title = visible[i - 1].strip()
        if title:
            headings.append(title)
            plain = re.sub(r"<[^>]*>", "", title)
            plain = re.sub(r"\[([^]]+)\]\([^)]*\)", r"\1", plain)
            slug = re.sub(r"[^\w\- ]", "", unescape(plain).lower()).replace(" ", "-")
            candidate, n = slug, 0
            while candidate in seen:
                n += 1
                candidate = f"{slug}-{n}"
            seen.add(candidate)
            ids.add(candidate)
    # Code examples are not links; inline code is not a reference label.
    prose = re.sub(r"(`+).*?\1", "", prose)
    definitions = {}
    for match in re.finditer(r"^ {0,3}\[([^]]+)\]:\s*(\S+)", prose, re.M):
        definitions[match[1].casefold()] = match[2].strip("<>")
    prose = re.sub(r"^ {0,3}\[[^]]+\]:.*$", "", prose, flags=re.M)
    links = html.links + re.findall(r"\[[^]\n]*\]\(<?([^\s)>]+)>?(?:\s+[^)]*)?\)", prose)
    for match in re.finditer(r"\[([^]\n]+)\](?:\[([^]\n]*)\])?(?!\()", prose):
        label = (match[2] or match[1]).casefold()
        if label in definitions:
            links.append(definitions[label])
        elif match[2] is not None:
            errors.append(f"undefined reference [{label}]")
    return ids, links, diagrams, headings, errors


def check_local(root, paths):
    """Return actionable errors and diagrams; no writes or network calls."""
    root = root.resolve()
    errors, parsed, diagrams = [], {}, []
    for missing in sorted(OWNERS - paths):
        errors.append(f"{missing}: missing documentation owner")
    for path in sorted(paths):
        if path in RETIRED or (path.endswith(".md") and any(
            part.lower() in {"archive", "archives"} for part in Path(path).parts
        )):
            errors.append(f"{path}: retired/archive surface must remain in Git history")
        if not path.endswith(".md"):
            continue
        parsed[path] = parse_markdown((root / path).read_text())
        _, _, blocks, headings, problems = parsed[path]
        diagrams.extend(dict(block, file=path) for block in blocks)
        errors.extend(f"{path}: {problem}" for problem in problems)
        # Enforce the explicit status surface, not guesses about semantic prose.
        if "Project status" in headings and path != "ROADMAP.md":
            errors.append(f"{path}: project-status owner must be ROADMAP.md")
    if "ROADMAP.md" in parsed and "Project status" not in parsed["ROADMAP.md"][3]:
        errors.append("ROADMAP.md: missing Project status heading")
    graph = {path: set() for path in parsed}
    for path, (_, links, _, _, _) in parsed.items():
        for link in links:
            url = urlsplit(link)
            if url.scheme or url.netloc:
                continue
            target = ((root / path).parent / unquote(url.path)).resolve() if url.path else root / path
            if not target.is_relative_to(root):
                errors.append(f"{path}: link escapes repository: {link}")
                continue
            if target.is_dir():
                target /= "README.md"
            relative = target.relative_to(root).as_posix()
            if relative not in paths:
                errors.append(f"{path}: missing link target: {link}")
                continue
            graph[path].add(relative)
            if url.fragment and relative in parsed and unquote(url.fragment) not in parsed[relative][0]:
                errors.append(f"{path}: missing anchor: {link}")
    reached, pending = set(), ["README.md"]
    while pending:
        path = pending.pop()
        if path not in reached:
            reached.add(path)
            pending.extend(graph.get(path, ()))
    for path in sorted(set(parsed) | {p for p in paths if p.startswith("assets/")}):
        if path not in reached:
            errors.append(f"{path}: unreachable from README.md")
    abi_path = root / "api/c-abi.json"
    if abi_path.is_file() and "docs/c-api.md" in parsed:
        schema = json.loads(abi_path.read_text())
        api = (root / "docs/c-api.md").read_text()
        for name, _, value in schema["constants"]:
            if name == "REPLAI_C_ABI_VERSION":
                versions = re.findall(r"\bABI (\d+)\b", api)
                if not versions or any(int(v) != value for v in versions):
                    errors.append(f"docs/c-api.md: ABI identity differs from schema ({value})")
            else:
                label = re.sub(r"^REPLAI_(?:EVENT_|ROLE_)?", "", name)
                values = re.findall(rf"^\| {re.escape(label)} \| (\d+) \|", api, re.M)
                if values != [str(value)]:
                    errors.append(f"docs/c-api.md: {name} must have one table value {value}; found {values}")
    else:
        errors.append("api/c-abi.json: missing ABI authority or C contract")
    return errors, diagrams


def check_mermaid(diagrams):
    """Run the real pinned Mermaid parser; missing tools fail the gate."""
    try:
        result = subprocess.run(
            ["node", str(ROOT / "tools/docs/check_mermaid.mjs")],
            input=json.dumps(diagrams), text=True, capture_output=True, timeout=60,
        )
    except (OSError, subprocess.TimeoutExpired) as error:
        return [f"tools/docs: Mermaid parser unavailable: {error}"]
    if result.returncode:
        return [result.stderr.strip() or "tools/docs: Mermaid parse failed"]
    return []


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=ROOT)
    args = parser.parse_args()
    errors, diagrams = check_local(args.root, inventory(args.root))
    errors.extend(check_mermaid(diagrams))
    if errors:
        raise SystemExit("\n".join(errors))
    print(f"PASS documentation links, reachability, owners, ABI tables and {len(diagrams)} Mermaid diagrams")


if __name__ == "__main__":
    main()
