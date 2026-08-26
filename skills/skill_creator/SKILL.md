---
name: skill-creator
version: "1.0.0"
description: |
  Agentic skill creation and management for Vibe/AutoDev.
  
  This meta-skill provides:
  - Skill definition templates (AMX-compliant, ECM-aware, Cline-compatible)
  - Validation against AGENTS.md rules and AMCX-1 specifications
  - Worktree-isolated skill development
  - RED-GREEN-REFACTOR test scaffolding
  - Integration with AMX Memory and ECM state
  
  Use this skill to create new agentic skills that comply with:
  - AutoDev AGENTS.md boundaries (ForgeCore, Cline fabric purity)
  - AMCX-1 v1.1 AMX/ECM requirements
  - Vibe tool permission model
  - Cline Skills/Hooks/Plugins architecture
model: "mistral-large-latest"
preferred_tools: [read_file, write_file, edit, grep, bash]
temperature: 0.0
---

## Skill Creator - Agentic Skill Development

**Purpose:** Meta-skill for creating, validating, and managing agentic skills in the AutoDev/Vibe ecosystem.

**Note:** This is a SELF-PROMPT skill. When invoked with `/skill-creator`, it guides the creation of new skills.

### Capabilities

1. **Skill Scaffolding**
   - Generate SKILL.md with proper frontmatter
   - Create skill implementation files
   - Set up test directories with RED phase tests
   - Initialize __init__.py files

2. **Validation**
   - Validate against AGENTS.md rules
   - Check AMCX-1 AMX compliance
   - Verify Cline fabric purity (no direct effects without tools)
   - Ensure tool permissions are respected

3. **Templates**
   - AMX Memory Skill template
   - ECM Collaboration Skill template
   - Evidence/Verification Skill template
   - Host Adapter Skill template
   - MCP Server Skill template

4. **Integration**
   - Auto-register with ECM state
   - Link to AMX memory patterns
   - Add to Vibe skill discovery

### Usage

#### Create a New Skill

```bash
# Invoke skill creator
/skill-creator

# Or programmatically
from skills.skill_creator.skill_creator import SkillCreator
creator = SkillCreator()
creator.create_skill(
    name="my-new-skill",
    skill_type="amx-memory",  # or "ecm", "evidence", "host-adapter", "mcp-server"
    description="My new skill description",
    worktree_path=".worktrees/my-skill-dev/"
)
```

#### Validate a Skill

```python
from skills.skill_creator.skill_creator import SkillCreator
creator = SkillCreator()

# Validate skill definition
report = creator.validate_skill(
    skill_path="skills/my-skill/SKILL.md",
    implementation_path="skills/my-skill/my_skill.py"
)
print(f"Valid: {report['valid']}")
print(f"Errors: {report['errors']}")
```

#### Use Templates

```python
from skills.skill_creator.skill_creator import SkillCreator, templates

# Get template for AMX memory skill
template = templates.AMX_MEMORY_SKILL_TEMPLATE
print(template)

# Create from template
creator.create_from_template(
    template_name="amx-memory",
    output_path="skills/my-amx-skill/",
    name="my-amx-skill",
    description="My AMX memory implementation"
)
```

### Skill Types

| Type | Purpose | Template | Test Scaffold |
|------|---------|----------|---------------|
| `amx-memory` | AMX v1.1 canonical memory | ✅ | ✅ |
| `ecm` | ECM collaboration state | ✅ | ✅ |
| `evidence` | Evidence/verification | ✅ | ✅ |
| `host-adapter` | Vibe tool extension | ✅ | ✅ |
| `mcp-server` | MCP resource server | ✅ | ✅ |
| `general` | General purpose | ✅ | ✅ |

### AGENTS.md Compliance Rules Enforced

1. **No ForgeCore Bypass**
   - Skills cannot execute filesystem/network/process effects without Vibe tools
   - Must use `read_file`, `write_file`, `bash`, etc. (not direct Python I/O)

2. **Cline Fabric Purity**
   - Direct implementation first (S0)
   - Skills for expertise, Hooks for safety, Plugins for tools
   - MCP only for external systems

3. **No Root Manifests**
   - No `Cargo.toml`, `package.json`, `pyproject.toml` at root
   - No `kotlin/gradle/libs.versions.toml` without ADR

4. **Tool Permissions**
   - Respects Vibe's config.toml allowlists
   - No bypassing permission boundaries

5. **Workspace Confinement**
   - Skills operate within worktree or designated directories
   - No writing outside allowed paths

### AMCX-1 Compliance Rules Enforced

1. **AMX Memory**
   - Canonical portable memory history
   - Required fields: origin, logical_identity, repository_scope, provenance, causal_ancestry, trust_validity_state, visibility, purpose, retraction_deletion_barriers, canonical_semantic_digest

2. **ECM State**
   - Entity-Collaboration Model
   - Tasks, Attempts, Roles, Messages, Leases, etc.
   - Schema validation against ecm-state-v1.json

3. **Evidence Records**
   - Verification results and freshness tracking
   - Schema validation against evidence-record-v1.json

### Validation Levels

```python
# Level 1: Basic structure
creator.validate_skill(skill_path, level=1)

# Level 2: AGENTS.md compliance
creator.validate_skill(skill_path, level=2)

# Level 3: AMCX-1 compliance
creator.validate_skill(skill_path, level=3)

# Level 4: Full integration (all levels)
creator.validate_skill(skill_path, level=4)  # default
```

### RED-GREEN-REFACTOR Test Scaffolding

When creating a skill, the skill-creator automatically generates:

```python
# In skills/my-skill/tests/test_my_skill.py

class TestMySkill(unittest.TestCase):
    """RED Phase: Tests written before implementation."""
    
    def test_skill_import(self):
        """Should be importable."""
        from skills.my_skill import MySkill
        self.assertTrue(hasattr(MySkill, 'expected_method'))
    
    def test_basic_functionality(self):
        """Should perform basic function."""
        # This will FAIL initially (RED)
        skill = MySkill()
        result = skill.expected_method()
        self.assertEqual(result, expected_value)
```

### Integration with Existing System

The skill-creator integrates with:
- **AMX Memory**: Stores skill definitions as AMX memories
- **ECM State**: Tracks skill development as tasks/attempts
- **Evidence Store**: Records validation results
- **Toolset Memory**: Learns from skill creation patterns

### Self-Prompt Invocation

When you use `/skill-creator`, this skill:
1. Prompts for skill type and details
2. Generates the skill scaffold
3. Creates RED phase tests
4. Validates against all compliance rules
5. Registers with ECM state
6. Provides next steps for GREEN phase implementation

---

**Examples:**

```python
# Create a new AMX memory skill
creator.create_skill(
    name="knowledge-base",
    skill_type="amx-memory",
    description="Knowledge base for project patterns",
    origin="vibe:skill-creator",
    logical_identity="knowledge-base-v1",
    purpose={
        "primary": "knowledge:storage",
        "task_class": "memory:amx"
    }
)

# Validate existing skill
report = creator.validate_skill("skills/knowledge-base/SKILL.md")
if not report["valid"]:
    print("Validation errors:", report["errors"])

# List available templates
templates = creator.list_templates()
print(f"Available: {templates}")
```

---

**Note:** This skill provides the self-prompt capability. Use `/skill-creator` to invoke it interactively.
