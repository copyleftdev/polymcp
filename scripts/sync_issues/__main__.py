from __future__ import annotations

import argparse
import sys
from pathlib import Path

from .github import DryRunClient, GitHubClient, GitHubError, RateLimitError
from .loader import IssueLoader
from .state import StateManager
from .sync import ConsoleOutput, IssueSyncer


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        prog="sync_issues",
        description="Sync issue definitions to GitHub",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  %(prog)s --dry-run                 Preview changes without modifying GitHub
  %(prog)s                           Sync all issues (skips unchanged)
  %(prog)s --force                   Re-sync all issues even if unchanged
  %(prog)s --reset                   Clear state and start fresh
  %(prog)s --status                  Show sync status without making changes
        """,
    )

    parser.add_argument(
        "--issues-dir",
        type=Path,
        default=Path(__file__).parent.parent.parent / ".github" / "issues",
        help="Path to issues directory (default: .github/issues)",
    )

    parser.add_argument(
        "--state-file",
        type=Path,
        default=Path(__file__).parent.parent.parent / ".github" / ".sync-state.json",
        help="Path to state file (default: .github/.sync-state.json)",
    )

    parser.add_argument(
        "--repo",
        type=str,
        default=None,
        help="GitHub repo in owner/name format (default: from git remote)",
    )

    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Preview changes without modifying GitHub",
    )

    parser.add_argument(
        "--force",
        action="store_true",
        help="Re-sync all issues even if unchanged",
    )

    parser.add_argument(
        "--reset",
        action="store_true",
        help="Clear sync state and start fresh",
    )

    parser.add_argument(
        "--status",
        action="store_true",
        help="Show sync status without making changes",
    )

    parser.add_argument(
        "--no-color",
        action="store_true",
        help="Disable colored output",
    )

    return parser.parse_args()


def detect_repo() -> tuple[str, str]:
    import subprocess

    result = subprocess.run(
        ["git", "remote", "get-url", "origin"],
        capture_output=True,
        text=True,
        check=False,
    )

    if result.returncode != 0:
        raise RuntimeError("Could not detect GitHub repo from git remote")

    url = result.stdout.strip()

    if url.startswith("git@github.com:"):
        path = url[len("git@github.com:") :]
    elif url.startswith("https://github.com/"):
        path = url[len("https://github.com/") :]
    else:
        raise RuntimeError(f"Unsupported remote URL format: {url}")

    path = path.removesuffix(".git")
    parts = path.split("/")

    if len(parts) != 2:
        raise RuntimeError(f"Invalid repo path: {path}")

    return parts[0], parts[1]


def show_status(state: StateManager, output: ConsoleOutput):
    summary = state.summary()
    output.info(f"Run ID: {state.state.run_id}")
    output.info(f"Started: {state.state.started_at.isoformat()}")

    if state.state.completed_at:
        output.success(f"Completed: {state.state.completed_at.isoformat()}")
    else:
        output.warning("Not completed")

    print()
    output.info(f"Completed: {summary.get('completed', 0)}")
    output.info(f"Pending:   {summary.get('pending', 0)}")
    output.info(f"Failed:    {summary.get('failed', 0)}")
    output.info(f"Skipped:   {summary.get('skipped', 0)}")

    failed = [r for r in state.state.records.values() if r.status.value == "failed"]
    if failed:
        print()
        output.error("Failed issues:")
        for record in failed:
            output.error(f"  {record.issue_id}: {record.error}")


def main() -> int:
    args = parse_args()
    output = ConsoleOutput(use_color=not args.no_color)

    print()
    output.info("pdfvec Issue Sync")
    print()

    if not args.issues_dir.exists():
        output.error(f"Issues directory not found: {args.issues_dir}")
        return 1

    try:
        loader = IssueLoader(args.issues_dir)
    except FileNotFoundError as e:
        output.error(str(e))
        return 1

    state = StateManager(args.state_file)

    if args.reset:
        state.reset()
        output.success("State cleared")
        return 0

    if args.status:
        show_status(state, output)
        return 0

    if args.repo:
        owner, repo = args.repo.split("/")
    else:
        try:
            owner, repo = detect_repo()
        except RuntimeError as e:
            output.error(str(e))
            output.info("Use --repo owner/name to specify manually")
            return 1

    output.info(f"Repository: {owner}/{repo}")
    output.info(f"Dry run: {'yes' if args.dry_run else 'no'}")
    print()

    client_cls = DryRunClient if args.dry_run else GitHubClient

    try:
        with client_cls(owner, repo) as client:
            syncer = IssueSyncer(client, loader, state, output)
            summary = syncer.sync(force=args.force)

    except RateLimitError as e:
        output.error(f"Rate limited. Try again after {e.reset_at}")
        return 1

    except GitHubError as e:
        output.error(f"GitHub API error: {e}")
        return 1

    print()
    output.success(f"Sync complete: {summary.get('completed', 0)} synced, {summary.get('skipped', 0)} skipped")

    if summary.get("failed", 0) > 0:
        output.warning(f"{summary['failed']} failed - run with --status for details")
        return 1

    return 0


if __name__ == "__main__":
    sys.exit(main())
