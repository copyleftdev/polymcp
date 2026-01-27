from __future__ import annotations

import enum
import hashlib
import json
from dataclasses import dataclass, field
from datetime import datetime
from pathlib import Path
from typing import Any


class SyncAction(enum.Enum):
    CREATE = "create"
    UPDATE = "update"
    SKIP = "skip"
    CLOSE = "close"


class SyncStatus(enum.Enum):
    PENDING = "pending"
    IN_PROGRESS = "in_progress"
    COMPLETED = "completed"
    FAILED = "failed"
    SKIPPED = "skipped"


@dataclass(frozen=True, slots=True)
class IssueRef:
    id: str
    file: Path

    @property
    def key(self) -> str:
        return self.id


@dataclass(slots=True)
class Issue:
    id: str
    title: str
    type: str
    status: str
    priority: str
    labels: list[str]
    milestone: str
    description: str
    acceptance_criteria: list[dict[str, Any]]
    epic: str | None = None
    depends_on: list[str] = field(default_factory=list)
    blocks: list[str] = field(default_factory=list)
    user_story: str | None = None
    technical_context: dict[str, Any] = field(default_factory=dict)
    state_machine: dict[str, Any] | None = None
    out_of_scope: list[str] = field(default_factory=list)
    open_questions: list[str] = field(default_factory=list)
    estimate: dict[str, Any] | None = None
    children: list[str] = field(default_factory=list)
    goals: list[str] = field(default_factory=list)
    design_principles: list[str] = field(default_factory=list)
    visual_mockups: dict[str, Any] | None = None
    interaction_patterns: dict[str, Any] | None = None
    responsive_behavior: dict[str, Any] | None = None
    source_file: Path | None = None
    github_number: int | None = None

    @classmethod
    def from_json(cls, data: dict[str, Any], source: Path | None = None) -> Issue:
        return cls(
            id=data["id"],
            title=data["title"],
            type=data["type"],
            status=data["status"],
            priority=data["priority"],
            labels=data.get("labels", []),
            milestone=data.get("milestone", ""),
            description=data.get("description", ""),
            acceptance_criteria=data.get("acceptance_criteria", []),
            epic=data.get("epic"),
            depends_on=data.get("depends_on", []),
            blocks=data.get("blocks", []),
            user_story=data.get("user_story"),
            technical_context=data.get("technical_context", {}),
            state_machine=data.get("state_machine"),
            out_of_scope=data.get("out_of_scope", []),
            open_questions=data.get("open_questions", []),
            estimate=data.get("estimate"),
            children=data.get("children", []),
            goals=data.get("goals", []),
            design_principles=data.get("design_principles", []),
            visual_mockups=data.get("visual_mockups"),
            interaction_patterns=data.get("interaction_patterns"),
            responsive_behavior=data.get("responsive_behavior"),
            source_file=source,
        )

    def content_hash(self) -> str:
        normalized = json.dumps(self._hashable_dict(), sort_keys=True, default=str)
        return hashlib.sha256(normalized.encode()).hexdigest()[:16]

    def _hashable_dict(self) -> dict[str, Any]:
        return {
            "id": self.id,
            "title": self.title,
            "type": self.type,
            "priority": self.priority,
            "labels": sorted(self.labels),
            "description": self.description,
            "acceptance_criteria": self.acceptance_criteria,
            "epic": self.epic,
            "depends_on": sorted(self.depends_on),
            "user_story": self.user_story,
            "technical_context": self.technical_context,
            "state_machine": self.state_machine,
            "out_of_scope": self.out_of_scope,
            "goals": self.goals,
        }

    def to_markdown(self) -> str:
        sections = [self._header(), self._body()]
        if self.user_story:
            sections.append(self._user_story_section())
        if self.acceptance_criteria:
            sections.append(self._acceptance_criteria_section())
        if self.state_machine:
            sections.append(self._state_machine_section())
        if self.technical_context:
            sections.append(self._technical_context_section())
        if self.visual_mockups:
            sections.append(self._visual_mockups_section())
        if self.out_of_scope:
            sections.append(self._out_of_scope_section())
        if self.open_questions:
            sections.append(self._open_questions_section())
        sections.append(self._metadata_section())
        return "\n\n".join(filter(None, sections))

    def _header(self) -> str:
        parts = [f"**{self.id}**"]
        if self.epic:
            parts.append(f"Epic: {self.epic}")
        if self.estimate:
            parts.append(f"Estimate: {self.estimate.get('points', '?')} pts")
        return " | ".join(parts)

    def _body(self) -> str:
        lines = []
        if self.goals:
            lines.append("### Goals\n")
            lines.extend(f"- {g}" for g in self.goals)
            lines.append("")
        lines.append(self.description)
        return "\n".join(lines)

    def _user_story_section(self) -> str:
        return f"### User Story\n\n> {self.user_story}"

    def _acceptance_criteria_section(self) -> str:
        lines = ["### Acceptance Criteria\n"]
        for ac in self.acceptance_criteria:
            lines.append(f"**{ac['id']}**")
            lines.append(f"- **Given** {ac['given']}")
            lines.append(f"- **When** {ac['when']}")
            lines.append(f"- **Then** {ac['then']}")
            if ac.get("notes"):
                lines.append(f"- *Note: {ac['notes']}*")
            lines.append("")
        return "\n".join(lines)

    def _state_machine_section(self) -> str:
        sm = self.state_machine
        lines = ["### State Machine\n"]
        lines.append(f"**Initial State:** `{sm['initial']}`\n")
        lines.append("#### States")
        for state in sm.get("states", []):
            terminal = " (terminal)" if state.get("terminal") else ""
            lines.append(f"- `{state['name']}`{terminal}: {state['description']}")
        lines.append("\n#### Transitions")
        for t in sm.get("transitions", []):
            guard = f" [{t['guard']}]" if t.get("guard") else ""
            lines.append(f"- `{t['from']}` → `{t['to']}`: {t['trigger']}{guard}")
        return "\n".join(lines)

    def _technical_context_section(self) -> str:
        tc = self.technical_context
        lines = ["### Technical Context\n"]
        if tc.get("crates"):
            lines.append(f"**Crates:** `{'`, `'.join(tc['crates'])}`\n")
        if tc.get("files"):
            lines.append("**Files:**")
            lines.extend(f"- `{f}`" for f in tc["files"])
            lines.append("")
        if tc.get("data_structures"):
            lines.append("#### Data Structures")
            for ds in tc["data_structures"]:
                lines.append(f"\n**`{ds['name']}`** - {ds.get('description', '')}")
                if ds.get("fields"):
                    for name, typ in ds["fields"].items():
                        lines.append(f"- `{name}`: {typ}")
        if tc.get("interfaces"):
            lines.append("\n#### Interfaces")
            for iface in tc["interfaces"]:
                lines.append(f"\n**`{iface['name']}`**")
                lines.append(f"```rust\n{iface['signature']}\n```")
                lines.append(iface.get("description", ""))
        if tc.get("error_cases"):
            lines.append("\n#### Error Cases")
            lines.extend(f"- {e}" for e in tc["error_cases"])
        if tc.get("performance_constraints"):
            pc = tc["performance_constraints"]
            lines.append("\n#### Performance Constraints")
            for k, v in pc.items():
                lines.append(f"- **{k.replace('_', ' ').title()}:** {v}")
        return "\n".join(lines)

    def _visual_mockups_section(self) -> str:
        lines = ["### Visual Mockups\n"]
        for name, mockup in self.visual_mockups.items():
            lines.append(f"#### {name.replace('_', ' ').title()}\n")
            if mockup.get("description"):
                lines.append(f"{mockup['description']}\n")
            if mockup.get("content"):
                content = mockup["content"]
                if isinstance(content, list):
                    lines.append("```")
                    lines.extend(content)
                    lines.append("```")
                elif isinstance(content, dict):
                    lines.append("```json")
                    lines.append(json.dumps(content, indent=2))
                    lines.append("```")
            if mockup.get("frames"):
                for frame in mockup["frames"]:
                    lines.append(f"\n**Frame {frame.get('frame', '?')}** ({frame.get('duration_ms', '?')}ms)")
                    if frame.get("content"):
                        lines.append("```")
                        lines.extend(frame["content"])
                        lines.append("```")
                    if frame.get("note"):
                        lines.append(f"*{frame['note']}*")
            if mockup.get("color_scheme"):
                lines.append("\n**Color Scheme:**")
                for elem, color in mockup["color_scheme"].items():
                    lines.append(f"- `{elem}`: {color}")
            lines.append("")
        return "\n".join(lines)

    def _out_of_scope_section(self) -> str:
        lines = ["### Out of Scope\n"]
        lines.extend(f"- {item}" for item in self.out_of_scope)
        return "\n".join(lines)

    def _open_questions_section(self) -> str:
        lines = ["### Open Questions\n"]
        lines.extend(f"- [ ] {q}" for q in self.open_questions)
        return "\n".join(lines)

    def _metadata_section(self) -> str:
        lines = ["---", f"*Source: `{self.source_file}`*" if self.source_file else ""]
        lines.append(f"*Content Hash: `{self.content_hash()}`*")
        return "\n".join(filter(None, lines))


@dataclass(slots=True)
class SyncRecord:
    issue_id: str
    action: SyncAction
    status: SyncStatus
    github_number: int | None = None
    content_hash: str | None = None
    error: str | None = None
    timestamp: datetime = field(default_factory=datetime.utcnow)

    def to_dict(self) -> dict[str, Any]:
        return {
            "issue_id": self.issue_id,
            "action": self.action.value,
            "status": self.status.value,
            "github_number": self.github_number,
            "content_hash": self.content_hash,
            "error": self.error,
            "timestamp": self.timestamp.isoformat(),
        }

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> SyncRecord:
        return cls(
            issue_id=data["issue_id"],
            action=SyncAction(data["action"]),
            status=SyncStatus(data["status"]),
            github_number=data.get("github_number"),
            content_hash=data.get("content_hash"),
            error=data.get("error"),
            timestamp=datetime.fromisoformat(data["timestamp"]),
        )


@dataclass(slots=True)
class SyncState:
    run_id: str
    started_at: datetime
    completed_at: datetime | None
    records: dict[str, SyncRecord] = field(default_factory=dict)

    def to_dict(self) -> dict[str, Any]:
        return {
            "run_id": self.run_id,
            "started_at": self.started_at.isoformat(),
            "completed_at": self.completed_at.isoformat() if self.completed_at else None,
            "records": {k: v.to_dict() for k, v in self.records.items()},
        }

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> SyncState:
        return cls(
            run_id=data["run_id"],
            started_at=datetime.fromisoformat(data["started_at"]),
            completed_at=datetime.fromisoformat(data["completed_at"]) if data.get("completed_at") else None,
            records={k: SyncRecord.from_dict(v) for k, v in data.get("records", {}).items()},
        )

    def is_completed(self, issue_id: str) -> bool:
        record = self.records.get(issue_id)
        return record is not None and record.status == SyncStatus.COMPLETED

    def needs_sync(self, issue: Issue) -> bool:
        record = self.records.get(issue.id)
        if record is None:
            return True
        if record.status != SyncStatus.COMPLETED:
            return True
        return record.content_hash != issue.content_hash()
