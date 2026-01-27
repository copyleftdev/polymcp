from __future__ import annotations

import os
import time
from dataclasses import dataclass
from functools import cached_property
from typing import Any, Iterator
from urllib.parse import urljoin

import httpx


class GitHubError(Exception):
    def __init__(self, message: str, status_code: int | None = None, response: dict | None = None):
        super().__init__(message)
        self.status_code = status_code
        self.response = response


class RateLimitError(GitHubError):
    def __init__(self, reset_at: int):
        super().__init__(f"Rate limited until {reset_at}")
        self.reset_at = reset_at


@dataclass(frozen=True, slots=True)
class GitHubIssue:
    number: int
    title: str
    body: str
    state: str
    labels: tuple[str, ...]
    milestone_number: int | None

    @classmethod
    def from_response(cls, data: dict[str, Any]) -> GitHubIssue:
        return cls(
            number=data["number"],
            title=data["title"],
            body=data.get("body") or "",
            state=data["state"],
            labels=tuple(lbl["name"] for lbl in data.get("labels", [])),
            milestone_number=data["milestone"]["number"] if data.get("milestone") else None,
        )


@dataclass(frozen=True, slots=True)
class GitHubMilestone:
    number: int
    title: str
    state: str

    @classmethod
    def from_response(cls, data: dict[str, Any]) -> GitHubMilestone:
        return cls(number=data["number"], title=data["title"], state=data["state"])


@dataclass(frozen=True, slots=True)
class GitHubLabel:
    name: str
    color: str
    description: str

    @classmethod
    def from_response(cls, data: dict[str, Any]) -> GitHubLabel:
        return cls(name=data["name"], color=data["color"], description=data.get("description") or "")


class GitHubClient:
    BASE_URL = "https://api.github.com"

    def __init__(self, owner: str, repo: str, token: str | None = None):
        self.owner = owner
        self.repo = repo
        self._token = token or os.environ.get("GITHUB_TOKEN")
        if not self._token:
            raise GitHubError("GITHUB_TOKEN environment variable required")
        self._client = httpx.Client(timeout=30.0)

    def close(self):
        self._client.close()

    def __enter__(self):
        return self

    def __exit__(self, *args):
        self.close()

    @cached_property
    def _headers(self) -> dict[str, str]:
        return {
            "Authorization": f"Bearer {self._token}",
            "Accept": "application/vnd.github+json",
            "X-GitHub-Api-Version": "2022-11-28",
        }

    def _url(self, path: str) -> str:
        return urljoin(self.BASE_URL, f"/repos/{self.owner}/{self.repo}/{path.lstrip('/')}")

    def _request(self, method: str, path: str, **kwargs) -> dict[str, Any] | list[Any]:
        url = self._url(path)
        response = self._client.request(method, url, headers=self._headers, **kwargs)

        if response.status_code == 403 and "rate limit" in response.text.lower():
            reset_at = int(response.headers.get("X-RateLimit-Reset", 0))
            raise RateLimitError(reset_at)

        if response.status_code >= 400:
            raise GitHubError(
                f"{method} {path} failed: {response.status_code}",
                status_code=response.status_code,
                response=response.json() if response.text else None,
            )

        return response.json() if response.text else {}

    def _paginate(self, path: str, params: dict | None = None) -> Iterator[dict[str, Any]]:
        params = {**(params or {}), "per_page": 100, "page": 1}
        while True:
            items = self._request("GET", path, params=params)
            if not items:
                break
            yield from items
            if len(items) < 100:
                break
            params["page"] += 1

    def get_issue(self, number: int) -> GitHubIssue | None:
        try:
            data = self._request("GET", f"issues/{number}")
            return GitHubIssue.from_response(data)
        except GitHubError as e:
            if e.status_code == 404:
                return None
            raise

    def list_issues(self, state: str = "all") -> Iterator[GitHubIssue]:
        for data in self._paginate("issues", {"state": state}):
            yield GitHubIssue.from_response(data)

    def create_issue(
        self,
        title: str,
        body: str,
        labels: list[str] | None = None,
        milestone: int | None = None,
    ) -> GitHubIssue:
        payload = {"title": title, "body": body}
        if labels:
            payload["labels"] = labels
        if milestone:
            payload["milestone"] = milestone
        data = self._request("POST", "issues", json=payload)
        return GitHubIssue.from_response(data)

    def update_issue(
        self,
        number: int,
        title: str | None = None,
        body: str | None = None,
        labels: list[str] | None = None,
        milestone: int | None = None,
        state: str | None = None,
    ) -> GitHubIssue:
        payload = {}
        if title is not None:
            payload["title"] = title
        if body is not None:
            payload["body"] = body
        if labels is not None:
            payload["labels"] = labels
        if milestone is not None:
            payload["milestone"] = milestone
        if state is not None:
            payload["state"] = state
        data = self._request("PATCH", f"issues/{number}", json=payload)
        return GitHubIssue.from_response(data)

    def list_milestones(self, state: str = "all") -> Iterator[GitHubMilestone]:
        for data in self._paginate("milestones", {"state": state}):
            yield GitHubMilestone.from_response(data)

    def create_milestone(self, title: str, description: str = "") -> GitHubMilestone:
        data = self._request("POST", "milestones", json={"title": title, "description": description})
        return GitHubMilestone.from_response(data)

    def list_labels(self) -> Iterator[GitHubLabel]:
        for data in self._paginate("labels"):
            yield GitHubLabel.from_response(data)

    def create_label(self, name: str, color: str, description: str = "") -> GitHubLabel:
        data = self._request("POST", "labels", json={"name": name, "color": color, "description": description})
        return GitHubLabel.from_response(data)

    def update_label(self, name: str, color: str | None = None, description: str | None = None) -> GitHubLabel:
        payload = {}
        if color is not None:
            payload["color"] = color
        if description is not None:
            payload["description"] = description
        data = self._request("PATCH", f"labels/{name}", json=payload)
        return GitHubLabel.from_response(data)


class DryRunClient:
    def __init__(self, owner: str, repo: str):
        self.owner = owner
        self.repo = repo
        self._issue_counter = 1000
        self._milestone_counter = 100
        self._issues: dict[int, GitHubIssue] = {}
        self._milestones: dict[str, GitHubMilestone] = {}
        self._labels: dict[str, GitHubLabel] = {}

    def close(self):
        pass

    def __enter__(self):
        return self

    def __exit__(self, *args):
        pass

    def get_issue(self, number: int) -> GitHubIssue | None:
        return self._issues.get(number)

    def list_issues(self, state: str = "all") -> Iterator[GitHubIssue]:
        yield from self._issues.values()

    def create_issue(
        self,
        title: str,
        body: str,
        labels: list[str] | None = None,
        milestone: int | None = None,
    ) -> GitHubIssue:
        self._issue_counter += 1
        issue = GitHubIssue(
            number=self._issue_counter,
            title=title,
            body=body,
            state="open",
            labels=tuple(labels or []),
            milestone_number=milestone,
        )
        self._issues[issue.number] = issue
        return issue

    def update_issue(
        self,
        number: int,
        title: str | None = None,
        body: str | None = None,
        labels: list[str] | None = None,
        milestone: int | None = None,
        state: str | None = None,
    ) -> GitHubIssue:
        existing = self._issues.get(number)
        if not existing:
            raise GitHubError(f"Issue {number} not found", status_code=404)
        issue = GitHubIssue(
            number=number,
            title=title if title is not None else existing.title,
            body=body if body is not None else existing.body,
            state=state if state is not None else existing.state,
            labels=tuple(labels) if labels is not None else existing.labels,
            milestone_number=milestone if milestone is not None else existing.milestone_number,
        )
        self._issues[number] = issue
        return issue

    def list_milestones(self, state: str = "all") -> Iterator[GitHubMilestone]:
        yield from self._milestones.values()

    def create_milestone(self, title: str, description: str = "") -> GitHubMilestone:
        self._milestone_counter += 1
        milestone = GitHubMilestone(number=self._milestone_counter, title=title, state="open")
        self._milestones[title] = milestone
        return milestone

    def list_labels(self) -> Iterator[GitHubLabel]:
        yield from self._labels.values()

    def create_label(self, name: str, color: str, description: str = "") -> GitHubLabel:
        label = GitHubLabel(name=name, color=color, description=description)
        self._labels[name] = label
        return label

    def update_label(self, name: str, color: str | None = None, description: str | None = None) -> GitHubLabel:
        existing = self._labels.get(name)
        if not existing:
            raise GitHubError(f"Label {name} not found", status_code=404)
        label = GitHubLabel(
            name=name,
            color=color if color is not None else existing.color,
            description=description if description is not None else existing.description,
        )
        self._labels[name] = label
        return label
