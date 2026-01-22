# wires

Lightweight local task tracker optimized for AI coding agents.

## Motivation: Memory for Agentic Development

AI coding agents face a unique challenge: **context loss**. Long conversations get compacted, sessions end, and without persistent memory, agents lose track of:
- What tasks were planned
- What's already completed
- What dependencies exist between tasks
- What should be worked on next

`wires` solves this by providing **persistent, structured memory** that survives context windows and session boundaries.

### The Problem

When working on complex, multi-step projects with AI agents:

1. **Context windows fill up** - Conversations get summarized, losing task details
2. **Sessions end** - You come back later and the agent has no memory of the plan
3. **Complex dependencies** - Hard to track what must complete before other work begins
4. **Lost progress** - Without records, agents duplicate work or lose track of what's done
5. **No continuity** - Each session feels like starting over

### The Solution

`wires` acts as **external memory** for agentic workflows:

- **Context recovery**: After compaction/restart, `wr ready` shows exactly what to work on next
- **Persistent plans**: Break complex work into wires with dependencies, stored locally
- **Progress tracking**: See what's done, in progress, or blocked at any time
- **Dependency awareness**: Agents know what must complete before starting new work
- **Session continuity**: Pick up exactly where you left off, even days later

Think of it as a lightweight issue tracker, but optimized for the unique constraints of AI development: local-only, JSON-native, dependency-aware, and designed to survive context loss.

## Overview

`wires` (command: `wr`) is a minimal task tracker designed for AI agents working on multi-step coding tasks. It stores tasks locally in a SQLite database (`.wires/wires.db`) and supports dependency tracking to help agents determine what to work on next.

**Key features:**
- JSON output for programmatic use (auto-detected when piped)
- Human-readable table output in terminals
- Dependency tracking with circular dependency prevention
- `ready` command to find unblocked tasks
- GraphViz DOT export for visualization
- Type-safe Rust implementation with compile-time validation

## Installation

### From source

```bash
git clone https://github.com/barooo/wires.git
cd wires
cargo build --release
cp target/release/wr ~/.local/bin/  # or wherever you keep binaries
```

## Quick Start

```bash
# Initialize in your project
cd my-project
wr init

# Create tasks for a multi-step feature
wr new "Setup database schema" -p 2
# {"id":"a1b2c3d", ...}

wr new "Implement API endpoints" -p 2
# {"id":"b2c3d4e", ...}

wr new "Add frontend UI" -p 1
# {"id":"c3d4e5f", ...}

wr new "Write integration tests" -p 1
# {"id":"d4e5f6a", ...}

# Set up dependencies (what must complete first)
wr dep b2c3d4e a1b2c3d  # API depends on database
wr dep c3d4e5f b2c3d4e  # UI depends on API
wr dep d4e5f6a c3d4e5f  # tests depend on UI

# Find what's ready to work on (only unblocked tasks)
wr ready
# Shows: a1b2c3d (database) - nothing blocks it

# Start working
wr start a1b2c3d

# Complete it
wr done a1b2c3d

# Check what's ready now
wr ready
# Shows: b2c3d4e (API) - database is done, so API is unblocked
```

## Commands

### Initialize
```bash
wr init
```
Creates `.wires/` directory with SQLite database.

### Create
```bash
wr new "Task title"
wr new "Task title" -d "Description"
wr new "Task title" -p 2  # priority (higher = more important)
```

### List
```bash
wr list                    # all wires
wr list -s todo            # filter by status (todo, in-progress, done, cancelled)
wr list -s in-progress
wr list -s done
wr list -f json            # force JSON output
wr list -f table           # force table output
```

### Show Details
```bash
wr show <id>
wr show <id> -f json
```

### Update
```bash
wr update <id> --title "New title"
wr update <id> --description "New description"
wr update <id> --status todo              # or TODO, in-progress, done, cancelled
wr update <id> --priority 3
```

### Status Shortcuts
```bash
wr start <id>   # set to IN_PROGRESS
wr done <id>    # set to DONE
wr cancel <id>  # set to CANCELLED
```

### Dependencies
```bash
wr dep <wire> <depends-on>    # wire depends on depends-on
wr undep <wire> <depends-on>  # remove dependency
```

### Find Ready Tasks
```bash
wr ready                  # tasks with no blocking dependencies
wr ready -f json
```

### Delete
```bash
wr rm <id>  # deletes wire and its dependency relationships
```

### Export Graph
```bash
wr graph                  # JSON format
wr graph -f json          # explicit JSON
wr graph -f dot           # GraphViz DOT format
```

## Output Formats

`wires` automatically detects whether output is going to a terminal or being piped:

- **Terminal (TTY):** Human-readable table format
- **Piped/Redirected:** JSON for programmatic parsing

Override with `-f json` or `-f table`.

### JSON Output Examples

```bash
# Create returns the new wire
$ wr new "Task" | jq .
{
  "id": "a1b2c3d",
  "title": "Task",
  "status": "TODO",
  ...
}

# List returns array
$ wr list -f json | jq '.[].title'

# Ready returns array of unblocked wires
$ wr ready -f json | jq '.[0].id'
```

## AI Agent Integration

`wires` is designed for AI coding agents that need persistent memory across context boundaries.

### Core Use Cases

1. **Multi-session projects**: Break complex work into wires at the start. When sessions end or context gets compacted, the plan persists. Next session picks up exactly where you left off.

2. **Context recovery**: After conversation compaction, `wr ready` immediately shows what's unblocked and ready to work on. No need to re-explain the plan or search through summarized history.

3. **Dependency management**: Set up dependencies between tasks. The agent always knows what can be worked on and what's blocked waiting for other work.

4. **Progress tracking**: At any point, see what's done, what's in progress, and what's remaining. Prevents duplicate work and maintains forward momentum.

5. **Long-running refactors**: For work spanning days or weeks, wires provide continuity. Each session starts with `wr ready` to find the next unblocked task.

### Recommended Workflow

```bash
# Session start: recover context
wr list              # see all tasks
wr ready             # what's unblocked and ready?
wr show <id>         # get details on next task

# Before starting work
wr start <id>

# During work: track new tasks discovered
wr new "Subtask found while implementing"
wr dep <new-task> <current-task>  # set up dependencies

# After completing
wr done <id>
wr ready             # what's next?

# End of session: verify state
wr list -s IN_PROGRESS   # anything left open?
```

### Integration Patterns

**Planning at task start:**
```bash
# Break down complex task into wires with dependencies
wr new "Setup database schema"
wr new "Implement API endpoints"
wr new "Write tests"
wr dep <api-id> <db-id>      # API depends on DB
wr dep <test-id> <api-id>    # tests depend on API
```

**Context recovery after compaction:**
```bash
# Conversation was compacted, what was I working on?
wr list -s IN_PROGRESS       # tasks marked started
wr ready                     # what can I work on now?
```

**Discovering work during implementation:**
```bash
# While implementing, realize you need to do X first
wr new "Missing dependency discovered"
wr dep <current-task> <new-task>  # current task now depends on new task
wr start <new-task>               # work on dependency first
```

### Why Local-Only?

`wires` deliberately avoids external services:
- **No API calls** - Works offline, no rate limits, instant responses
- **No authentication** - Zero setup friction
- **Privacy** - Your tasks stay on your machine
- **Reliability** - No service outages or network issues
- **Speed** - SQLite queries are microseconds, not HTTP roundtrips

This makes it ideal for agentic workflows where the agent needs reliable, fast access to task state without external dependencies.

### Real-World Experience

This tool (and its predecessor `beads`) has been used extensively in AI-assisted development sessions. Key observations:

- **Context compaction survival**: When conversations exceed context windows and get summarized, wires are the *only* reliable way to recover the exact work plan. Summaries lose task details and dependencies.

- **Session boundaries**: Real projects span multiple sessions over days or weeks. Without persistent task tracking, each session wastes time re-establishing context. With wires, you start each session with `wr ready` and immediately know what to work on.

- **Dependency clarity**: Complex refactors involve many interdependent tasks. Dependencies in wires prevent the agent from attempting blocked work and ensure correct implementation order.

- **Progress visibility**: During long-running work, `wr list -s DONE` provides a sense of progress and prevents the agent from re-doing completed work after context loss.

- **Discovered work**: Implementation often uncovers additional required tasks. Adding them as wires with dependencies ensures they don't get forgotten after compaction.

The value isn't just "task tracking" - it's **persistent memory that survives the limitations of AI conversation contexts**.

### Error Handling

Errors are output to stderr in the same format as regular output:
- Terminal: `Error: message`
- Piped: `{"error": "message"}`

Exit code is non-zero on error.

## Data Storage

- Database: `.wires/db.sqlite`
- Add `.wires/` to `.gitignore` (local-only tracking)
- Database uses WAL mode for concurrent access

## Status Values

- `TODO` / `todo` - Not started
- `IN_PROGRESS` / `in-progress` - Currently being worked on
- `DONE` / `done` - Completed
- `CANCELLED` / `cancelled` - Abandoned

Status arguments accept both kebab-case (`in-progress`) and uppercase (`IN_PROGRESS`) formats.

## Type Safety (v0.2.0+)

`wires` leverages Rust's type system for reliability:

- **WireId newtype**: Prevents mixing wire IDs with arbitrary strings, validates 7-character hex format
- **Status/Format enums**: Compile-time validation of CLI arguments via clap's `ValueEnum`
- **WireError enum**: Pattern-matchable domain errors (NotARepository, WireNotFound, CircularDependency, AlreadyInitialized)
- **Wire::new() constructor**: Validates non-empty titles, handles ID generation and timestamps

These improvements eliminate entire classes of runtime errors and provide better error messages when things go wrong.

## License

MIT
