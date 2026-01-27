from __future__ import annotations

import re
import sys
from dataclasses import dataclass, field
from typing import TYPE_CHECKING, Protocol

from .models import Issue, SyncAction, SyncStatus

if TYPE_CHECKING:
    from .github import GitHubClient, GitHubIssue, GitHubLabel, GitHubMilestone
    from .loader import IssueIndex, IssueLoader, LabelDef, MilestoneDef
    from .state import StateManager


class OutputHandler(Protocol):
    def info(self, msg: str) -> None: ...
    def success(self, msg: str) -> None: ...
    def warning(self, msg: str) -> None: ...
    def error(self, msg: str) -> None: ...
    def progress(self, current: int, total: int, msg: str) -> None: ...


class ConsoleOutput:
    RESET = "\033[0m"
    BOLD = "\033[1m"
    DIM = "\033[2m"
    GREEN = "\033[32m"
    YELLOW = "\033[33m"
    RED = "\033[31m"
    CYAN = "\033[36m"

    def __init__(self, use_color: bool = True):
        self.use_color = use_color and sys.stdout.isatty()

    def _c(self, code: str, text: str) -> str:
        return f"{code}{text}{self.RESET}" if self.use_color else text

    def info(self, msg: str):
        print(f"  {self._c(self.DIM, '•')} {msg}")

    def success(self, msg: str):
        print(f"  {self._c(self.GREEN, '✓')} {msg}")

    def warning(self, msg: str):
        print(f"  {self._c(self.YELLOW, '!')} {msg}")

    def error(self, msg: str):
        print(f"  {self._c(self.RED, '✗')} {msg}")

    def progress(self, current: int, total: int, msg: str):
        pct = (current / total) * 100 if total else 0
        bar_width = 20
        filled = int(bar_width * current / total) if total else 0
        bar = "█" * filled + "░" * (bar_width - filled)
        print(f"\r  {self._c(self.CYAN, bar)} {pct:5.1f}% {msg:<50}", end="", flush=True)
        if current == total:
            print()


@dataclass
class SyncPlan:
    creates: list[Issue] = field(default_factory=list)
    updates: list[tuple[Issue, int]] = field(default_factory=list)
    skips: list[tuple[Issue, int]] = field(default_factory=list)

    @property
    def total(self) -> int:
        return len(self.creates) + len(self.updates) + len(self.skips)

    @property
    def actions_needed(self) -> int:
        return len(self.creates) + len(self.updates)


class IssueSyncer:
    ISSUE_ID_PATTERN = re.compile(r"\*\*([A-Z]+-\d+)\*\*")

    def __init__(
        self,
        client: GitHubClient,
        loader: IssueLoader,
        state: StateManager,
        output: OutputHandler | None = None,
    ):
        self.client = client
        self.loader = loader
        self.state = state
        self.output = output or ConsoleOutput()
        self._milestones: dict[str, int] = {}
        self._labels: set[str] = set()
        self._existing_issues: dict[str, tuple[int, str]] = {}

    def sync(self, force: bool = False) -> dict[str, int]:
        self._load_github_state()
        issues = self.loader.load_all_issues()
        index = self._build_index(issues)
        ordered = self.loader.topological_sort(issues)

        plan = self._build_plan(ordered, force)
        self._print_plan(plan)

        if plan.actions_needed == 0:
            self.output.success("Nothing to sync")
            return self.state.summary()

        self._ensure_labels()
        self._ensure_milestones()
        self._execute_plan(plan, index)

        self.state.mark_run_completed()
        return self.state.summary()

    def _load_github_state(self):
        self.output.info("Loading GitHub state...")

        for milestone in self.client.list_milestones():
            self._milestones[milestone.title] = milestone.number

        for label in self.client.list_labels():
            self._labels.add(label.name)

        for gh_issue in self.client.list_issues():
            match = self.ISSUE_ID_PATTERN.search(gh_issue.body)
            if match:
                issue_id = match.group(1)
                self._existing_issues[issue_id] = (gh_issue.number, gh_issue.body)

        self.output.success(f"Found {len(self._existing_issues)} existing issues")

    def _build_index(self, issues: list[Issue]) -> IssueIndex:
        from .loader import IssueIndex
        return IssueIndex(issues)

    def _build_plan(self, issues: list[Issue], force: bool) -> SyncPlan:
        plan = SyncPlan()

        for issue in issues:
            existing = self._existing_issues.get(issue.id)

            if existing is None:
                plan.creates.append(issue)
            elif force or self._content_changed(issue, existing[1]):
                plan.updates.append((issue, existing[0]))
            else:
                plan.skips.append((issue, existing[0]))

        return plan

    def _content_changed(self, issue: Issue, existing_body: str) -> bool:
        current_hash = issue.content_hash()
        if f"Content Hash: `{current_hash}`" in existing_body:
            return False
        return self.state.needs_sync(issue)

    def _print_plan(self, plan: SyncPlan):
        self.output.info(f"Plan: {len(plan.creates)} create, {len(plan.updates)} update, {len(plan.skips)} skip")

    def _ensure_labels(self):
        defined = self.loader.load_labels()
        for label_def in defined:
            if label_def.name not in self._labels:
                self.client.create_label(label_def.name, label_def.color, label_def.description)
                self._labels.add(label_def.name)
                self.output.success(f"Created label: {label_def.name}")

    def _ensure_milestones(self):
        defined = self.loader.load_milestones()
        for ms_def in defined:
            if ms_def.title not in self._milestones:
                milestone = self.client.create_milestone(ms_def.title, ms_def.description)
                self._milestones[ms_def.title] = milestone.number
                self.output.success(f"Created milestone: {ms_def.title}")

    def _execute_plan(self, plan: SyncPlan, index: IssueIndex):
        total = plan.actions_needed
        current = 0

        for issue in plan.creates:
            current += 1
            self.output.progress(current, total, f"Creating {issue.id}")
            self._create_issue(issue, index)

        for issue, number in plan.updates:
            current += 1
            self.output.progress(current, total, f"Updating {issue.id}")
            self._update_issue(issue, number, index)

        for issue, number in plan.skips:
            self.state.mark_started(issue.id, SyncAction.SKIP)
            self.state.mark_skipped(issue.id, number)

    def _create_issue(self, issue: Issue, index: IssueIndex):
        self.state.mark_started(issue.id, SyncAction.CREATE)

        try:
            labels = self._resolve_labels(issue)
            milestone = self._milestones.get(issue.milestone)
            body = self._render_body(issue, index)

            gh_issue = self.client.create_issue(
                title=f"[{issue.type.upper()}] {issue.title}",
                body=body,
                labels=labels,
                milestone=milestone,
            )

            self._existing_issues[issue.id] = (gh_issue.number, gh_issue.body)
            self.state.mark_completed(issue.id, gh_issue.number, issue.content_hash())
            self.output.success(f"Created #{gh_issue.number}: {issue.id}")

        except Exception as e:
            self.state.mark_failed(issue.id, str(e))
            self.output.error(f"Failed {issue.id}: {e}")
            raise

    def _update_issue(self, issue: Issue, number: int, index: IssueIndex):
        self.state.mark_started(issue.id, SyncAction.UPDATE)

        try:
            labels = self._resolve_labels(issue)
            milestone = self._milestones.get(issue.milestone)
            body = self._render_body(issue, index)

            gh_issue = self.client.update_issue(
                number=number,
                title=f"[{issue.type.upper()}] {issue.title}",
                body=body,
                labels=labels,
                milestone=milestone,
            )

            self._existing_issues[issue.id] = (gh_issue.number, gh_issue.body)
            self.state.mark_completed(issue.id, gh_issue.number, issue.content_hash())
            self.output.success(f"Updated #{gh_issue.number}: {issue.id}")

        except Exception as e:
            self.state.mark_failed(issue.id, str(e))
            self.output.error(f"Failed {issue.id}: {e}")
            raise

    def _resolve_labels(self, issue: Issue) -> list[str]:
        return [lbl for lbl in issue.labels if lbl in self._labels]

    def _render_body(self, issue: Issue, index: IssueIndex) -> str:
        body = issue.to_markdown()

        if issue.depends_on:
            refs = self._render_issue_refs("Depends On", issue.depends_on)
            body = f"{refs}\n\n{body}"

        if issue.blocks:
            refs = self._render_issue_refs("Blocks", issue.blocks)
            body = f"{refs}\n\n{body}"

        if issue.children:
            refs = self._render_issue_refs("Child Issues", issue.children)
            body = f"{body}\n\n{refs}"

        return body

    def _render_issue_refs(self, label: str, issue_ids: list[str]) -> str:
        refs = []
        for issue_id in issue_ids:
            existing = self._existing_issues.get(issue_id)
            if existing:
                refs.append(f"#{existing[0]}")
            else:
                refs.append(f"`{issue_id}`")
        return f"**{label}:** {', '.join(refs)}"
