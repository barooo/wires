# wires

Lightweight local task tracker optimized for AI coding agents.

## Overview

`wires` (command: `wr`) is a minimal task tracker designed for AI agents working on multi-step coding tasks. It stores tasks locally in a SQLite database (`.wires/db.sqlite`) and supports dependency tracking to help agents determine what to work on next.

**Key features:**
- JSON output for programmatic use (auto-detected when piped)
- Human-readable table output in terminals
- Dependency tracking with circular dependency prevention
- `ready` command to find unblocked tasks
- GraphViz DOT export for visualization

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

# Create some tasks
wr new "Implement user authentication"
wr new "Add login page" -d "Frontend login form"
wr new "Write tests" -p 2  # priority 2

# Set up dependencies (login page depends on auth)
wr dep <login-page-id> <auth-id>

# Find what's ready to work on
wr ready

# Start working on a task
wr start <id>

# Mark complete
wr done <id>
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
wr list -s TODO            # filter by status
wr list -s IN_PROGRESS
wr list -s DONE
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
wr update <id> --status TODO
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

`wires` is designed for AI coding agents that need to:

1. **Track multi-step work:** Break complex tasks into trackable pieces
2. **Handle dependencies:** Know what must complete before starting something
3. **Resume context:** After compaction/restart, check `wr ready` for next steps
4. **Parse output:** JSON output enables reliable extraction of IDs and status

### Recommended Workflow

```bash
# At session start
wr ready -f json  # what can I work on?

# Before starting work
wr start <id>

# After completing
wr done <id>
wr ready -f json  # what's next?
```

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

- `TODO` - Not started
- `IN_PROGRESS` - Currently being worked on
- `DONE` - Completed
- `CANCELLED` - Abandoned

## License

MIT
