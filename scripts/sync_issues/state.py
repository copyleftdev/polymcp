from __future__ import annotations

import json
import uuid
from datetime import datetime
from pathlib import Path
from typing import TYPE_CHECKING

from .models import SyncRecord, SyncState, SyncStatus

if TYPE_CHECKING:
    from .models import Issue, SyncAction


class StateManager:
    def __init__(self, state_file: Path):
        self.state_file = state_file
        self._state: SyncState | None = None

    @property
    def state(self) -> SyncState:
        if self._state is None:
            self._state = self._load_or_create()
        return self._state

    def _load_or_create(self) -> SyncState:
        if self.state_file.exists():
            data = json.loads(self.state_file.read_text())
            return SyncState.from_dict(data)
        return SyncState(
            run_id=str(uuid.uuid4())[:8],
            started_at=datetime.utcnow(),
            completed_at=None,
        )

    def save(self):
        self.state_file.parent.mkdir(parents=True, exist_ok=True)
        self.state_file.write_text(json.dumps(self.state.to_dict(), indent=2, default=str))

    def reset(self):
        if self.state_file.exists():
            self.state_file.unlink()
        self._state = None

    def start_new_run(self) -> str:
        self._state = SyncState(
            run_id=str(uuid.uuid4())[:8],
            started_at=datetime.utcnow(),
            completed_at=None,
        )
        self.save()
        return self.state.run_id

    def mark_started(self, issue_id: str, action: SyncAction):
        self.state.records[issue_id] = SyncRecord(
            issue_id=issue_id,
            action=action,
            status=SyncStatus.IN_PROGRESS,
        )
        self.save()

    def mark_completed(self, issue_id: str, github_number: int, content_hash: str):
        record = self.state.records.get(issue_id)
        if record:
            record.status = SyncStatus.COMPLETED
            record.github_number = github_number
            record.content_hash = content_hash
            record.timestamp = datetime.utcnow()
        self.save()

    def mark_failed(self, issue_id: str, error: str):
        record = self.state.records.get(issue_id)
        if record:
            record.status = SyncStatus.FAILED
            record.error = error
            record.timestamp = datetime.utcnow()
        self.save()

    def mark_skipped(self, issue_id: str, github_number: int | None = None):
        record = self.state.records.get(issue_id)
        if record:
            record.status = SyncStatus.SKIPPED
            record.github_number = github_number
            record.timestamp = datetime.utcnow()
        self.save()

    def mark_run_completed(self):
        self.state.completed_at = datetime.utcnow()
        self.save()

    def is_completed(self, issue_id: str) -> bool:
        return self.state.is_completed(issue_id)

    def needs_sync(self, issue: Issue) -> bool:
        return self.state.needs_sync(issue)

    def get_github_number(self, issue_id: str) -> int | None:
        record = self.state.records.get(issue_id)
        return record.github_number if record else None

    def pending_count(self) -> int:
        return sum(1 for r in self.state.records.values() if r.status == SyncStatus.PENDING)

    def completed_count(self) -> int:
        return sum(1 for r in self.state.records.values() if r.status == SyncStatus.COMPLETED)

    def failed_count(self) -> int:
        return sum(1 for r in self.state.records.values() if r.status == SyncStatus.FAILED)

    def summary(self) -> dict[str, int]:
        counts = {s: 0 for s in SyncStatus}
        for record in self.state.records.values():
            counts[record.status] += 1
        return {s.value: c for s, c in counts.items()}
