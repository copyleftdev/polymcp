from __future__ import annotations

import json
from collections.abc import Sequence
from dataclasses import dataclass
from pathlib import Path
from typing import Iterator

from .models import Issue


@dataclass(frozen=True, slots=True)
class LabelDef:
    name: str
    color: str
    description: str


@dataclass(frozen=True, slots=True)
class MilestoneDef:
    title: str
    description: str


class IssueLoader:
    def __init__(self, issues_dir: Path):
        self.issues_dir = issues_dir
        self._validate_structure()

    def _validate_structure(self):
        required = ["_schema.json", "_labels.json", "_milestones.json"]
        missing = [f for f in required if not (self.issues_dir / f).exists()]
        if missing:
            raise FileNotFoundError(f"Missing required files: {missing}")

    def load_labels(self) -> list[LabelDef]:
        data = json.loads((self.issues_dir / "_labels.json").read_text())
        return [LabelDef(name=lbl["name"], color=lbl["color"], description=lbl.get("description", "")) for lbl in data["labels"]]

    def load_milestones(self) -> list[MilestoneDef]:
        data = json.loads((self.issues_dir / "_milestones.json").read_text())
        return [MilestoneDef(title=ms["title"], description=ms.get("description", "")) for ms in data["milestones"]]

    def load_index(self) -> dict:
        index_file = self.issues_dir / "_index.json"
        if not index_file.exists():
            raise FileNotFoundError("_index.json not found")
        return json.loads(index_file.read_text())

    def iter_issue_files(self) -> Iterator[Path]:
        for subdir in ["epics", "stories"]:
            path = self.issues_dir / subdir
            if path.exists():
                yield from path.rglob("*.json")

    def load_issue(self, path: Path) -> Issue:
        data = json.loads(path.read_text())
        return Issue.from_json(data, source=path.relative_to(self.issues_dir))

    def load_all_issues(self) -> list[Issue]:
        issues = []
        for path in self.iter_issue_files():
            issues.append(self.load_issue(path))
        return issues

    def topological_sort(self, issues: Sequence[Issue]) -> list[Issue]:
        by_id = {i.id: i for i in issues}
        visited: set[str] = set()
        result: list[Issue] = []

        def visit(issue_id: str):
            if issue_id in visited:
                return
            visited.add(issue_id)
            issue = by_id.get(issue_id)
            if issue is None:
                return
            for dep_id in issue.depends_on:
                visit(dep_id)
            if issue.epic:
                visit(issue.epic)
            result.append(issue)

        for issue in issues:
            visit(issue.id)

        return result


class IssueIndex:
    def __init__(self, issues: Sequence[Issue]):
        self._by_id: dict[str, Issue] = {i.id: i for i in issues}
        self._by_title: dict[str, Issue] = {}
        self._children: dict[str, list[str]] = {}

        for issue in issues:
            normalized = self._normalize_title(issue.title)
            self._by_title[normalized] = issue

            if issue.epic:
                self._children.setdefault(issue.epic, []).append(issue.id)

    @staticmethod
    def _normalize_title(title: str) -> str:
        return title.lower().strip()

    def get(self, issue_id: str) -> Issue | None:
        return self._by_id.get(issue_id)

    def find_by_title(self, title: str) -> Issue | None:
        return self._by_title.get(self._normalize_title(title))

    def children_of(self, epic_id: str) -> list[Issue]:
        child_ids = self._children.get(epic_id, [])
        return [self._by_id[cid] for cid in child_ids if cid in self._by_id]

    def all_ids(self) -> set[str]:
        return set(self._by_id.keys())

    def __len__(self) -> int:
        return len(self._by_id)

    def __iter__(self) -> Iterator[Issue]:
        yield from self._by_id.values()
