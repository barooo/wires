# Agent Instructions

`wires` is a local task tracker designed for AI coding agents. Use it to persist work plans across context boundaries.

## When to Use

- Multi-step tasks that may span sessions or survive context compaction
- Work with dependencies (task B requires task A to complete first)
- Any time you need to recover "what was I working on?"

## Core Workflow

```bash
# Start of session: what's ready to work on?
wr ready

# Before starting a task
wr start <id>

# After completing
wr done <id>

# Check what's next
wr ready
```

## Creating Tasks

```bash
wr new "Task title"
wr new "Task title" -d "Description with details"
wr new "Task title" -p 2  # higher priority = more important
```

## Dependencies

```bash
wr dep <task> <depends-on>  # task cannot start until depends-on is done
```

Dependencies are enforced by `wr ready`—it only shows tasks with no incomplete blockers.

## Quick Reference

| Command | Purpose |
|---------|---------|
| `wr init` | Initialize in current directory |
| `wr ready` | Show unblocked tasks |
| `wr list` | Show all tasks |
| `wr show <id>` | Task details |
| `wr start <id>` | Mark in-progress |
| `wr done <id>` | Mark complete |
| `wr dep <a> <b>` | a depends on b |

## Key Principle

After context loss, `wr ready` tells you exactly what to work on next. Use it liberally.
