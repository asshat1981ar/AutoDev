---
name: vibe-setup
description: |
  Load this skill when setting up, configuring, or troubleshooting Vibe in a project.
  
  Provides comprehensive guidance on:
  - Vibe installation and configuration
  - Project setup and initialization
  - Trusted folder and permission management
  - Skill discovery and loading
  - MCP server configuration
  - Integration with AutoDev's AGENTS.md rules
  
  Use this when the user needs help with Vibe environment setup.
model: mistral-medium-3.5
user-invocable: true
---

# Vibe Setup Guide

**Purpose:** Complete guide to Vibe installation, configuration, and project integration.

## Quick Start

### 1. Installation

```bash
# Clone Vibe (if not using pre-installed)
git clone https://github.com/mistralai/vibe.git
cd vibe

# Install dependencies
pip install -e .

# Or use the Termux launcher
node scripts/termux-kanban.mjs
```

### 2. Basic Configuration

Create `~/.vibe/config.toml`:

```toml
[general]
# Model configuration
model = "mistral-medium-3.5"
provider = "mistral"

[permissions]
# Allowed tools
allowed_tools = ["read_file", "write_file", "edit", "grep", "bash", "python"]

[skills]
# Skill paths (discovery order: first wins)
skill_paths = [
    ".vibe/skills",
    ".agents/skills",
    "~/.vibe/skills"
]
```

### 3. Project Initialization

```bash
# Create project structure
mkdir -p .vibe/skills .vibe/logs

# Create initial SKILL.md for project
cat > .vibe/skills/project/SKILL.md << 'EOF'
---
name: project-guidance
description: Project-specific guidance for AutoDev
model: mistral-medium-3.5
---

# Project Guidance

**Purpose:** Provide project-specific context and rules.

## AutoDev Rules

Follow all rules in AGENTS.md:
- Trusted execution kernel: crates/forge-core
- Control-plane server: crates/autodev-server
- Kotlin modules: kotlin/*
- Python fabric: install.py, bootstrap_cline_mcp.py
- Termux launcher: scripts/termux-kanban.mjs

## Verification Gates

All changes must pass:
- cargo fmt --check (Rust)
- cargo clippy (Rust)
- cargo test (Rust)
- ./gradlew test (Kotlin)
- python -m unittest discover (Python)
- node --check (Node)
- python scripts/check_harness_drift.py
EOF
```

## Configuration Reference

### config.toml Structure

```toml
[general]
# Model settings
model = "mistral-medium-3.5"
provider = "mistral"
temperature = 0.7

# Behavior
verbose = true
interactive = true

[permissions]
# Tool whitelist (null = all tools allowed)
allowed_tools = ["read_file", "write_file", "edit", "grep", "bash"]

# Denied tools (overrides allowed_tools)
denied_tools = []

# File permissions
allowed_paths = [
    "/data/data/com.termux/files/home/AutoDev"
]
denied_paths = [
    "/etc",
    "/root",
    "/home"
]

[skills]
# Skill discovery paths
skill_paths = [
    ".vibe/skills",
    ".agents/skills",
    "~/.vibe/skills"
]

# Auto-load skills
auto_load = ["meta-creator", "vibe-setup"]

# User-invocable skills (via /command)
user_invocable = true

[logging]
# Log level: debug, info, warning, error
level = "info"
file = ".vibe/logs/vibe.log"

[mcp]
# MCP server configuration
enabled = true
host = "127.0.0.1"
port = 8080

# Trusted MCP servers
trusted_servers = [
    "engram-mcp",
    "github-app",
    "linear"
]

[features]
# Experimental features
experimental_tools = false
skill_hot_reload = true
```

### Environment Variables

```bash
# Model provider
export VIBE_MODEL="mistral-medium-3.5"
export VIBE_PROVIDER="mistral"

# Paths
export VIBE_HOME="~/.vibe"
export VIBE_CONFIG="~/.vibe/config.toml"

# Logging
export VIBE_LOG_LEVEL="info"
export VIBE_LOG_FILE=".vibe/logs/vibe.log"

# Performance
export VIBE_TIMEOUT="300"
export VIBE_MAX_TOKENS="4096"
```

## Trusted Folder Setup

### For AutoDev Project

Vibe must trust the AutoDev directory to load skills and access files.

```bash
# Option 1: Add to config.toml
cat >> ~/.vibe/config.toml << 'EOF'

[general]
trusted_folders = [
    "/data/data/com.termux/files/home/AutoDev"
]
EOF

# Option 2: Environment variable
mkdir -p /data/data/com.termux/files/home/AutoDev/.vibe
cat > /data/data/com.termux/files/home/AutoDev/.vibe/config.toml << 'EOF'
[general]
trusted_folders = [
    "/data/data/com.termux/files/home/AutoDev"
]
EOF

# Option 3: Symlink to user config
ln -s /data/data/com.termux/files/home/AutoDev/.vibe/config.toml ~/.vibe/config-autodev.toml
```

### Verify Trusted Status

```bash
# Check if Vibe can access the project
python -c "
from vibe.cli import CLI
cli = CLI()
print('Trusted folders:', cli.config.get('trusted_folders', []))
"

# Test skill loading
python -c "
from vibe.skills import SkillLoader
loader = SkillLoader()
skills = loader.discover_skills()
print('Discovered skills:', list(skills.keys()))
"
```

## Skill Management

### Creating Skills

**Recommended Structure:**

```
.vibe/skills/
├── skill-name/
│   ├── SKILL.md          # Required: Instructions
│   ├── __init__.py       # Optional: Package init
│   ├── implementation.py # Optional: Code
│   └── tests/
│       └── test_*.py     # Optional: Tests
```

**SKILL.md Format:**

```markdown
---
name: skill-name
version: "1.0.0"
description: |
  Load this skill when [specific condition]
  
  This skill provides [capability]
model: mistral-medium-3.5
user-invocable: true
---

# Skill Name

**Purpose:** [Clear purpose statement]

## When to Load

- [ ] Condition 1
- [ ] Condition 2

## Capabilities

- Capability 1
- Capability 2
```

### Loading Skills

```bash
# Manual load
/vibe-setup

# Auto-load (in config.toml)
[skills]
auto_load = ["vibe-setup", "meta-creator"]

# List loaded skills
/skills list

# Reload all skills
/reload

# Load specific skill
/load vibe-setup
```

## MCP Server Setup

### Local MCP Server

```python
#!/usr/bin/env python3
"""Local MCP Server for AutoDev."""

from vibe.mcp import MCPServer
import json

class AutoDevMCPServer(MCPServer):
    """AutoDev-specific MCP server."""
    
    def __init__(self):
        super().__init__("autodev-mcp")
        self.resources = {
            "memory://autodev": self.handle_memory,
            "ecm://autodev": self.handle_ecm,
        }
    
    def handle_memory(self, operation, params):
        """Handle memory operations."""
        # Implement AMX-compliant memory
        pass
    
    def handle_ecm(self, operation, params):
        """Handle ECM operations."""
        # Implement ECM tracking
        pass

if __name__ == "__main__":
    server = AutoDevMCPServer()
    server.start(host="127.0.0.1", port=8080)
```

### Connect to Existing MCP Servers

```toml
# config.toml
[mcp]
enabled = true

[mcp.servers]

# Engram MCP
["engram-mcp"]
command = "python"
args = ["-m", "engram_mcp"]
env = {VIBE_PROJECT = "AutoDev"}

# GitHub App
["github-app"]
command = "python"
args = ["-m", "github_app_mcp"]

# Linear
["linear"]
command = "npx"
args = ["@linear/linear-mcp"]
```

## Permission Management

### Tool Permissions

Vibe enforces tool permissions at runtime. All tools must be explicitly allowed.

**Allowed Tools in AutoDev:**

```toml
[permissions]
allowed_tools = [
    # File operations
    "read_file",
    "write_file", 
    "edit",
    
    # Search
    "grep",
    
    # Shell
    "bash",
    
    # Git
    "git",
    
    # Process
    "python",
    
    # Network (restricted)
    "web_fetch",
    "web_search"
]
```

### Path Restrictions

```toml
[permissions]
# Allow access to specific paths
allowed_paths = [
    "/data/data/com.termux/files/home/AutoDev"
]

# Deny access to sensitive paths
denied_paths = [
    "/etc",
    "/root", 
    "/home",
    "/proc",
    "/sys",
    "/dev"
]

# Allow patterns
allowed_patterns = [
    "*.py",
    "*.md",
    "*.toml",
    "*.json",
    "*.yaml",
    "*.yml"
]

# Deny patterns
denied_patterns = [
    "*.pem",
    "*.key",
    "*.secret",
    ".env",
    "secrets/*"
]
```

## AutoDev-Specific Setup

### Project Structure

```
AutoDev/
├── .vibe/
│   ├── config.toml          # Vibe config
│   ├── skills/              # Project skills
│   │   └── autodev/         # AutoDev-specific skills
│   │       ├── SKILL.md
│   │       └── __init__.py
│   └── logs/                # Vibe logs
├── AGENTS.md               # Project rules
├── crates/                 # Rust workspace
├── kotlin/                 # Kotlin modules
├── scripts/                # Utility scripts
└── docs/                   # Documentation
```

### AGENTS.md Integration

Vibe must respect AutoDev's AGENTS.md rules:

1. **Trusted Execution:** Only `crates/forge-core` can execute untrusted I/O
2. **Workspace Confined:** All operations must be workspace-confined
3. **No Direct Execution:** No direct filesystem/network/process execution without `AuthorizationGrant`
4. **Package Manager Rules:** No root manifests, use only documented commands

**Vibe Config for AGENTS.md Compliance:**

```toml
[general]
trusted_folders = [
    "/data/data/com.termux/files/home/AutoDev"
]

[permissions]
allowed_tools = ["read_file", "write_file", "edit", "grep", "bash"]

# Deny dangerous operations
denied_tools = [
    "rm",
    "mv",
    "cp",
    "chmod",
    "chown"
]

# Restrict to workspace
denied_paths = [
    "/",
    "/etc",
    "/root",
    "/home",
    "/usr",
    "/var",
    "/tmp"
]

allowed_paths = [
    "/data/data/com.termux/files/home/AutoDev"
]
```

### Workspace-Confined Execution

```python
import os
from pathlib import Path

class WorkspaceGuard:
    """Ensure all operations stay within workspace."""
    
    WORKSPACE = Path("/data/data/com.termux/files/home/AutoDev")
    
    @staticmethod
    def check_path(path: str) -> bool:
        """Check if path is within workspace."""
        full_path = Path(path).resolve()
        workspace = WorkspaceGuard.WORKSPACE.resolve()
        
        try:
            full_path.relative_to(workspace)
            return True
        except ValueError:
            return False
    
    @staticmethod
    def guard_read(file_path: str) -> str:
        """Guarded read operation."""
        if not WorkspaceGuard.check_path(file_path):
            raise PermissionError(f"Cannot read outside workspace: {file_path}")
        
        with open(file_path, 'r') as f:
            return f.read()
    
    @staticmethod
    def guard_write(file_path: str, content: str) -> None:
        """Guarded write operation."""
        if not WorkspaceGuard.check_path(file_path):
            raise PermissionError(f"Cannot write outside workspace: {file_path}")
        
        with open(file_path, 'w') as f:
            f.write(content)
```

## Testing Vibe Setup

### Verification Checklist

```bash
# 1. Vibe can be invoked
vibe --version

# 2. Configuration loads
python -c "from vibe.cli import CLI; cli = CLI(); print(cli.config)"

# 3. Skills can be discovered
python -c "from vibe.skills import SkillLoader; print(SkillLoader().discover_skills())"

# 4. Trusted folder works
python -c "
from vibe.permissions import PermissionChecker
checker = PermissionChecker()
print('Can read AGENTS.md:', checker.can_read('/data/data/com.termux/files/home/AutoDev/AGENTS.md'))
"

# 5. Tools are available
python -c "
from vibe.tools import ToolRegistry
registry = ToolRegistry()
print('Available tools:', list(registry.tools.keys()))
"

# 6. MCP server can start
python -c "
from vibe.mcp import MCPServer
server = MCPServer('test')
print('MCP server created successfully')
"
```

## Troubleshooting

### Common Issues

**1. Skills not loading**

```bash
# Check skill paths
python -c "
from vibe.skills import SkillLoader
loader = SkillLoader()
print('Skill paths:', loader.skill_paths)
print('Discovered skills:', list(loader.discover_skills().keys()))
"

# Check file permissions
ls -la .vibe/skills/*/SKILL.md

# Check config
cat ~/.vibe/config.toml | grep skill_paths
```

**2. Permission denied errors**

```bash
# Check trusted folders
python -c "
from vibe.config import Config
config = Config.load()
print('Trusted folders:', config.get('trusted_folders', []))
"

# Add current directory to trusted folders
python -c "
import os
from vibe.config import Config
config = Config.load()
config['trusted_folders'] = config.get('trusted_folders', []) + [os.getcwd()]
config.save()
"
```

**3. Tool not found errors**

```bash
# Check allowed tools
python -c "
from vibe.config import Config
config = Config.load()
print('Allowed tools:', config.get('allowed_tools', []))
"

# Add tool to allowed list
python -c "
from vibe.config import Config
config = Config.load()
config['allowed_tools'] = config.get('allowed_tools', []) + ['new_tool']
config.save()
"
```

**4. MCP server connection failures**

```bash
# Check MCP config
python -c "
from vibe.config import Config
config = Config.load()
print('MCP config:', config.get('mcp', {}))
"

# Test MCP server manually
python -c "
from vibe.mcp import MCPServer
server = MCPServer('test')
# Manually test endpoints
"
```

### Debug Mode

Enable verbose logging for troubleshooting:

```bash
# Set log level
export VIBE_LOG_LEVEL=debug

# Or in config.toml
[logging]
level = "debug"

# View logs
cat .vibe/logs/vibe.log

# Tail logs
watch -n 1 cat .vibe/logs/vibe.log
```

## Performance Optimization

### Caching

```toml
[cache]
# Enable skill caching
enabled = true

# Cache TTL in seconds
ttl = 3600

# Max cache size
max_size = 1000
```

### Resource Limits

```toml
[limits]
# Max tokens per response
max_tokens = 4096

# Max tool calls per response
max_tool_calls = 20

# Timeout in seconds
timeout = 300

# Max concurrent requests
max_concurrent = 5
```

## Advanced Configuration

### Multiple Projects

```toml
# ~/.vibe/config.toml

[projects]

[projects.autodev]
path = "/data/data/com.termux/files/home/AutoDev"
trusted = true
auto_load_skills = ["autodev-guidance", "rust-tools"]

[projects.other]
path = "/other/project"
trusted = false
auto_load_skills = []
```

### Environment-Specific Config

```toml
# ~/.vibe/config.toml
[general]
# Base configuration

# Override for development
environment = "development"

# ~/.vibe/config-dev.toml
[general]
environment = "development"
[permissions]
allowed_tools = ["read_file", "write_file", "edit", "grep", "bash", "python"]

# ~/.vibe/config-prod.toml
[general]
environment = "production"
[permissions]
allowed_tools = ["read_file", "grep"]
```

## Migration Guide

### From Vibe 0.x to 1.x

```bash
# Backup old config
cp ~/.vibe/config.toml ~/.vibe/config.toml.bak

# Create new config
cat > ~/.vibe/config.toml << 'EOF'
[general]
model = "mistral-medium-3.5"
provider = "mistral"

[permissions]
allowed_tools = ["read_file", "write_file", "edit", "grep", "bash"]

[skills]
skill_paths = [".vibe/skills", "~/.vibe/skills"]
EOF

# Migrate skills
mkdir -p ~/.vibe/skills
cp -r ~/.vibe/old-skills/* ~/.vibe/skills/
```

### Skill Format Migration

```python
#!/usr/bin/env python3
"""Migrate old skill format to new format."""

import os
import re
from pathlib import Path

def migrate_skill(skill_dir):
    """Migrate a skill directory to new format."""
    skill_md = skill_dir / "SKILL.md"
    
    if not skill_md.exists():
        return
    
    content = skill_md.read_text()
    
    # Add frontmatter if missing
    if not content.startswith("---"):
        # Extract name from directory
        name = skill_dir.name
        description = f"Load this skill for {name} functionality"
        
        new_content = f"""---
name: {name}
description: {description}
model: mistral-medium-3.5
---

{content}"""
        skill_md.write_text(new_content)
        print(f"Migrated: {skill_dir}")

# Migrate all skills
for skill_dir in Path("~/.vibe/skills").glob("*"):
    if skill_dir.is_dir():
        migrate_skill(skill_dir)
```

## Security Checklist

- [ ] Trusted folders configured correctly
- [ ] Denied paths include sensitive directories
- [ ] Allowed tools are minimal and necessary
- [ ] MCP servers are from trusted sources
- [ ] Skills have clear descriptions and conditions
- [ ] All file operations use path validation
- [ ] No hardcoded credentials or secrets
- [ ] Error messages don't leak sensitive information

## References

- [Vibe GitHub](https://github.com/mistralai/vibe)
- [Vibe Documentation](https://docs.vibe.mistral.ai)
- [MCP Specification](https://github.com/modelcontextprotocol/spec)
- [AutoDev AGENTS.md](/data/data/com.termux/files/home/AutoDev/AGENTS.md)
- [AutoDev README.md](/data/data/com.termux/files/home/AutoDev/README.md)
