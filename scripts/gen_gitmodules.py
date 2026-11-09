#!/usr/bin/env python3
"""Regenerate .gitmodules from the submodule gitlinks recorded in the index.

Every gitlink (mode 160000) under dataset/contracts/ gets an entry whose URL is
taken from the `origin` remote of the checked-out submodule. Mirror/proxy
prefixes and SSH URLs are normalized to plain public HTTPS URLs.

With --check, also verify via the GitHub API (`gh`) that each upstream
repository is public and still contains the pinned commit.

Usage (from the repository root):
    python3 scripts/gen_gitmodules.py [--check]
"""

import json
import re
import subprocess
import sys


def sh(*args, cwd=None, check=True):
    return subprocess.run(
        args, cwd=cwd, check=check, capture_output=True, text=True
    ).stdout.strip()


def gitlinks():
    out = subprocess.run(
        ["git", "ls-files", "-s", "-z"], check=True, capture_output=True, text=True
    ).stdout
    for entry in out.split("\0"):
        if entry.startswith("160000 "):
            meta, path = entry.split("\t", 1)
            yield path, meta.split()[1]


def normalize(url):
    url = re.sub(r"^https?://ghfast\.top/", "", url)  # strip GitHub proxy mirror
    m = re.match(r"^git@([^:]+):(.+)$", url)  # SSH -> HTTPS
    if m:
        url = f"https://{m.group(1)}/{m.group(2)}"
    if not re.match(r"^[a-z]+://", url):  # e.g. "github.com/org/repo" left by the mirror
        url = "https://" + url
    url = url.rstrip("/")
    return url if url.endswith(".git") else url + ".git"


def check_github(url, sha):
    m = re.match(r"^https://github\.com/([^/]+)/([^/]+?)(?:\.git)?$", url)
    if not m:
        return "skipped (not GitHub)"
    repo = f"{m.group(1)}/{m.group(2)}"
    r = subprocess.run(
        ["gh", "api", f"repos/{repo}"], capture_output=True, text=True
    )
    if r.returncode != 0:
        return "REPO NOT FOUND"
    if json.loads(r.stdout).get("private"):
        return "REPO IS PRIVATE"
    r = subprocess.run(
        ["gh", "api", f"repos/{repo}/commits/{sha}", "--jq", ".sha"],
        capture_output=True,
        text=True,
    )
    return "ok" if r.returncode == 0 else "COMMIT NOT FOUND UPSTREAM"


def main():
    do_check = "--check" in sys.argv
    entries, problems = [], []
    for path, sha in gitlinks():
        url = sh("git", "-C", path, "remote", "get-url", "origin", check=False)
        if not url:
            problems.append((path, "no origin remote"))
            continue
        entries.append((path, normalize(url), sha))

    entries.sort()
    with open(".gitmodules", "w") as f:
        for path, url, _ in entries:
            name = path.replace('"', '\\"')
            f.write(f'[submodule "{name}"]\n\tpath = {path}\n\turl = {url}\n')
    print(f"Wrote .gitmodules with {len(entries)} submodules")

    if do_check:
        for path, url, sha in entries:
            status = check_github(url, sha)
            if status != "ok":
                problems.append((path, f"{status}: {url} @ {sha[:12]}"))

    for path, msg in problems:
        print(f"[!] {path}: {msg}")
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main())
