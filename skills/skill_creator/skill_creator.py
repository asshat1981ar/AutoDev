"""
Skill Creator - Meta-skill for creating agentic skills

This module provides the core implementation for the skill-creator meta-skill.
It enables the creation, validation, and management of agentic skills that comply
with AutoDev AGENTS.md rules and AMCX-1 v1.1 specifications.

Integration:
- AMX Memory: Stores skill definitions as portable memory records
- ECM State: Tracks skill development as tasks/attempts/roles
- Vibe Tools: Uses only allowed tools (read_file, write_file, edit, grep, bash)
- Cline Fabric: Respects Skills/Hooks/Plugins boundaries

Author: AutoDev Skill Creator
Version: 1.0.0
"""

import hashlib
import json
import os
import re
import unittest
from dataclasses import dataclass, field, asdict
from datetime import datetime, timezone
from enum import Enum
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple, Union

from . import templates


class SkillValidationError(Exception):
    """Raised when skill validation fails."""
    def __init__(self, message: str, errors: Optional[List[str]] = None,
                 level: int = 4):
        self.message = message
        self.errors = errors or []
        self.level = level
        super().__init__(self.message)

    def __str__(self) -> str:
        if self.errors:
            error_list = "\n  - ".join(self.errors)
            return f"{self.message}\n  - {error_list}"
        return self.message


class SkillCreationError(Exception):
    """Raised when skill creation fails."""
    pass


class SkillType(Enum):
    AMX_MEMORY = "amx-memory"
    ECM = "ecm"
    EVIDENCE = "evidence"
    HOST_ADAPTER = "host-adapter"
    MCP_SERVER = "mcp-server"
    ENGRAM_MCP = "engram-mcp"
    GENERAL = "general"


class ValidationLevel(Enum):
    BASIC = 1
    AGENTS_MD = 2
    AMCX1 = 3
    FULL = 4


class ComplianceStatus(Enum):
    PASS = "pass"
    WARN = "warn"
    FAIL = "fail"


@dataclass
class ValidationResult:
    valid: bool
    level: int
    errors: List[str] = field(default_factory=list)
    warnings: List[str] = field(default_factory=list)
    info: List[str] = field(default_factory=list)
    compliance: Dict[str, ComplianceStatus] = field(default_factory=dict)
    
    def to_dict(self) -> Dict[str, Any]:
        return asdict(self)


@dataclass
class SkillMetadata:
    name: str
    version: str = "1.0.0"
    description: str = ""
    skill_type: str = "general"
    origin: str = "vibe:skill-creator"
    logical_identity: str = ""
    primary_purpose: str = ""
    task_class: str = ""
    model: str = "mistral-large-latest"
    preferred_tools: List[str] = field(default_factory=lambda: [
        "read_file", "write_file", "edit", "grep", "bash"
    ])
    temperature: float = 0.0
    repository_scope: Dict[str, str] = field(default_factory=dict)
    
    def to_dict(self) -> Dict[str, Any]:
        return asdict(self)


@dataclass
class TemplateInfo:
    name: str
    description: str
    skill_type: SkillType
    files: List[str]
    required_fields: List[str] = field(default_factory=list)
    
    def to_dict(self) -> Dict[str, Any]:
        return asdict(self)


# AMX Memory Schema (AMCX-1 v1.1)
AMX_MEMORY_REQUIRED_FIELDS = [
    "schema_version",
    "origin",
    "logical_identity",
    "repository_scope",
    "provenance",
    "causal_ancestry",
    "trust_validity_state",
    "visibility",
    "purpose",
    "retraction_deletion_barriers",
    "canonical_semantic_digest",
]

ECM_REQUIRED_ENTITIES = [
    "task",
    "attempt",
    "role",
    "role_lease",
    "message",
    "artifact_reference",
    "evidence_reference",
    "memory_binding",
    "ContextView",
]

AGENTS_MD_RULES = {
    "no_forgecore_bypass": {
        "description": "Skills cannot bypass ForgeCore boundaries",
        "banned": ["os.system", "subprocess.run", "subprocess.Popen", "eval(", "exec("],
        "severity": "fail"
    },
    "cline_fabric_purity": {
        "description": "Skills must use Vibe tools, not direct I/O",
        "banned": ["builtins.open", "io.open", "os.remove", "os.rename", "shutil.rmtree"],
        "severity": "fail"
    },
    "no_root_manifests": {
        "description": "No root Cargo.toml, package.json, pyproject.toml",
        "forbidden_files": ["Cargo.toml", "package.json", "pyproject.toml", "requirements.txt"],
        "severity": "fail"
    },
    "tool_permissions": {
        "description": "Must respect Vibe tool permissions",
        "allowed_tools": ["read_file", "write_file", "edit", "grep", "bash", "ask_user_question", "skill", "task", "todo", "web_fetch", "web_search"],
        "severity": "warn"
    },
    "workspace_confinement": {
        "description": "Skills must operate within designated directories",
        "severity": "fail"
    }
}


class SkillCreator:
    """
    Main class for creating, validating, and managing agentic skills.
    """
    TEMPLATES: Dict[str, TemplateInfo] = {}
    
    def __init__(self, base_path: Optional[str] = None, worktree_path: Optional[str] = None):
        self.base_path = Path(base_path or os.getcwd())
        self.worktree_path = Path(worktree_path or ".")
        self._initialize_templates()
    
    def _initialize_templates(self) -> None:
        self.TEMPLATES = {
            "amx-memory": TemplateInfo(
                name="amx-memory",
                description="AMX v1.1 canonical portable memory skill",
                skill_type=SkillType.AMX_MEMORY,
                files=["SKILL.md", "amx_memory.py", "__init__.py", "tests/test_amx_memory.py"],
                required_fields=["origin", "logical_identity", "primary_purpose", "task_class"]
            ),
            "ecm": TemplateInfo(
                name="ecm",
                description="ECM collaboration state skill",
                skill_type=SkillType.ECM,
                files=["SKILL.md", "ecm.py", "__init__.py", "tests/test_ecm.py"],
                required_fields=["origin", "logical_identity"]
            ),
            "evidence": TemplateInfo(
                name="evidence",
                description="Evidence and verification skill",
                skill_type=SkillType.EVIDENCE,
                files=["SKILL.md", "evidence.py", "__init__.py", "tests/test_evidence.py"],
                required_fields=["origin", "logical_identity"]
            ),
            "host-adapter": TemplateInfo(
                name="host-adapter",
                description="Vibe tool extension / host adapter skill",
                skill_type=SkillType.HOST_ADAPTER,
                files=["SKILL.md", "host_adapter.py", "__init__.py", "tests/test_host_adapter.py"],
                required_fields=["origin", "logical_identity", "adapter_type"]
            ),
            "mcp-server": TemplateInfo(
                name="mcp-server",
                description="MCP resource server skill",
                skill_type=SkillType.MCP_SERVER,
                files=["SKILL.md", "mcp_server.py", "__init__.py", "tests/test_mcp_server.py"],
                required_fields=["origin", "logical_identity", "resource_uri"]
            ),
            "engram-mcp": TemplateInfo(
                name="engram-mcp",
                description="Engram MCP Server - Persistent memory with MCP interface",
                skill_type=SkillType.ENGRAM_MCP,
                files=["SKILL.md", "engram_mcp.py", "__init__.py", "tests/test_engram_mcp.py"],
                required_fields=["origin", "logical_identity"]
            ),
            "general": TemplateInfo(
                name="general",
                description="General purpose skill",
                skill_type=SkillType.GENERAL,
                files=["SKILL.md", "skill.py", "__init__.py", "tests/test_skill.py"],
                required_fields=["origin", "logical_identity"]
            ),
        }
    
    def list_templates(self) -> List[Dict[str, Any]]:
        return [t.to_dict() for t in self.TEMPLATES.values()]
    
    def get_template(self, template_name: str) -> Optional[TemplateInfo]:
        return self.TEMPLATES.get(template_name)
    
    def get_template_contents(self, template_name: str) -> Dict[str, str]:
        template = self.get_template(template_name)
        if not template:
            raise SkillValidationError(
                f"Template '{template_name}' not found",
                errors=[f"Available: {', '.join(self.TEMPLATES.keys())}"]
            )
        template_mapping = {
            "amx-memory": templates.AMX_MEMORY_SKILL_MD_TEMPLATE,
            "ecm": templates.ECM_SKILL_MD_TEMPLATE,
            "evidence": templates.EVIDENCE_SKILL_MD_TEMPLATE,
            "host-adapter": templates.HOST_ADAPTER_SKILL_MD_TEMPLATE,
            "mcp-server": templates.MCP_SERVER_SKILL_MD_TEMPLATE,
            "engram-mcp": templates.ENGRAM_MCP_SKILL_MD_TEMPLATE,
            "general": templates.GENERAL_SKILL_MD_TEMPLATE,
        }
        contents = {}
        if template_name in template_mapping:
            contents["SKILL.md"] = template_mapping[template_name]
        impl_mapping = {
            "amx-memory": templates.AMX_MEMORY_IMPL_TEMPLATE,
            "ecm": templates.ECM_IMPL_TEMPLATE,
            "evidence": templates.EVIDENCE_IMPL_TEMPLATE,
            "host-adapter": templates.HOST_ADAPTER_IMPL_TEMPLATE,
            "mcp-server": templates.MCP_SERVER_IMPL_TEMPLATE,
            "engram-mcp": templates.ENGRAM_MCP_IMPL_TEMPLATE,
            "general": templates.GENERAL_IMPL_TEMPLATE,
        }
        if template_name in impl_mapping:
            contents[f"{template_name.replace('-', '_')}.py"] = impl_mapping[template_name]
        contents["__init__.py"] = templates.INIT_TEMPLATE
        test_mapping = {
            "amx-memory": templates.AMX_MEMORY_TEST_TEMPLATE,
            "ecm": templates.ECM_TEST_TEMPLATE,
            "evidence": templates.EVIDENCE_TEST_TEMPLATE,
            "host-adapter": templates.HOST_ADAPTER_TEST_TEMPLATE,
            "mcp-server": templates.MCP_SERVER_TEST_TEMPLATE,
            "engram-mcp": templates.ENGRAM_MCP_TEST_TEMPLATE,
            "general": templates.GENERAL_TEST_TEMPLATE,
        }
        if template_name in test_mapping:
            contents[f"tests/test_{template_name.replace('-', '_')}.py"] = test_mapping[template_name]
        return contents

    def create_skill(
        self,
        name: str,
        skill_type: Union[str, SkillType] = SkillType.GENERAL,
        description: str = "",
        output_path: Optional[str] = None,
        worktree_path: Optional[str] = None,
        origin: str = "vibe:skill-creator",
        logical_identity: Optional[str] = None,
        primary_purpose: str = "",
        task_class: str = "",
        version: str = "1.0.0",
        model: str = "mistral-large-latest",
        preferred_tools: Optional[List[str]] = None,
        temperature: float = 0.0,
        **kwargs
    ) -> Path:
        if isinstance(skill_type, str):
            try:
                skill_type = SkillType(skill_type)
            except ValueError:
                raise SkillValidationError(
                    f"Invalid skill_type: {skill_type}",
                    errors=[f"Valid: {[t.value for t in SkillType]}"]
                )
        logical_identity = logical_identity or f"{name}-v1"
        preferred_tools = preferred_tools or ["read_file", "write_file", "edit", "grep", "bash"]
        template_name = skill_type.value
        template_info = self.get_template(template_name)
        if not template_info:
            raise SkillCreationError(f"No template for: {skill_type}")
        if output_path:
            skill_dir = Path(output_path)
        else:
            dir_name = name.replace('-', '_')
            if worktree_path:
                skill_dir = Path(worktree_path) / "skills" / dir_name
            else:
                skill_dir = self.base_path / "skills" / dir_name
        self._validate_skill_name(name)
        self._create_directory_structure(skill_dir)
        template_contents = self.get_template_contents(template_name)
        impl_file_mapping = {
            "amx-memory": "amx_memory",
            "ecm": "ecm",
            "evidence": "evidence",
            "host-adapter": "host_adapter",
            "mcp-server": "mcp_server",
            "engram-mcp": "engram_mcp",
            "general": self._to_file_name(name),
        }
        impl_file = impl_file_mapping.get(template_name, self._to_file_name(name))
        substitutions = {
            "name": name,
            "version": version,
            "description": description,
            "origin": origin,
            "logical_identity": logical_identity,
            "primary_purpose": primary_purpose,
            "task_class": task_class,
            "model": model,
            "preferred_tools": json.dumps(preferred_tools),
            "temperature": temperature,
            "class_name": self._to_class_name(name),
            "file_name": impl_file,
            "timestamp": datetime.now(timezone.utc).isoformat(),
            **kwargs
        }
        created_files = []
        for filename, content in template_contents.items():
            rendered = self._apply_substitutions(content, substitutions)
            filepath = skill_dir / filename
            filepath.parent.mkdir(parents=True, exist_ok=True)
            with open(filepath, 'w', encoding='utf-8') as f:
                f.write(rendered)
            created_files.append(filepath)
        test_dir = skill_dir / "tests"
        test_dir.mkdir(parents=True, exist_ok=True)
        test_file_name = f"test_{impl_file}.py"
        test_path = test_dir / test_file_name
        if not test_path.exists():
            self._create_red_tests(skill_dir, name, skill_type, substitutions)
        validation_result = self.validate_skill(skill_dir / "SKILL.md", level=4)
        if not validation_result.valid and validation_result.errors:
            raise SkillValidationError(
                f"Validation failed for '{name}'",
                errors=validation_result.errors,
                level=4
            )
        if skill_type in [SkillType.ECM, SkillType.AMX_MEMORY]:
            self._register_with_ecm(skill_dir, name, skill_type, substitutions)
        if skill_type == SkillType.AMX_MEMORY:
            self._store_in_amx(skill_dir, name, substitutions)
        return skill_dir

    def create_from_template(
        self,
        template_name: str,
        output_path: str,
        name: str,
        description: str = "",
        **kwargs
    ) -> Path:
        template = self.get_template(template_name)
        if not template:
            raise SkillValidationError(
                f"Template '{template_name}' not found",
                errors=[f"Available: {', '.join(self.TEMPLATES.keys())}"]
            )
        return self.create_skill(
            name=name,
            skill_type=template.skill_type,
            description=description,
            output_path=output_path,
            **kwargs
        )

    def _create_directory_structure(self, skill_dir: Path) -> None:
        skill_dir.mkdir(parents=True, exist_ok=True)
        (skill_dir / "tests").mkdir(parents=True, exist_ok=True)
        (skill_dir / "docs").mkdir(parents=True, exist_ok=True)
        (skill_dir / "tests" / "__init__.py").touch(exist_ok=True)
        (skill_dir / "docs" / "__init__.py").touch(exist_ok=True)

    def _validate_skill_name(self, name: str) -> None:
        if not name:
            raise SkillValidationError("Skill name cannot be empty")
        if not re.match(r'^[a-zA-Z0-9][a-zA-Z0-9_-]*$', name):
            raise SkillValidationError(
                f"Invalid skill name: {name}",
                errors=[
                    "Must start with alphanumeric",
                    "Only alphanumeric, hyphen, underscore allowed",
                    "Cannot end with hyphen or underscore"
                ]
            )
        reserved = ["skill", "creator", "skill-creator", "vibe", "autodev"]
        if name.lower() in reserved:
            raise SkillValidationError(
                f"Reserved name: {name}",
                errors=["Cannot use reserved name"]
            )

    def _to_class_name(self, name: str) -> str:
        return ''.join(word.title() for word in re.split(r'[-_]', name))

    def _to_file_name(self, name: str) -> str:
        return name.replace('-', '_')

    def _apply_substitutions(self, content: str, substitutions: Dict[str, Any]) -> str:
        for key, value in substitutions.items():
            if value is None:
                value = ""
            if not isinstance(value, str):
                value = str(value)
            placeholder = "{" + key + "}"
            content = content.replace(placeholder, value)
        # Convert {{ to { and }} to } for literal braces
        content = content.replace("{{", "{").replace("}}", "}")
        return content

    def _create_red_tests(
        self,
        skill_dir: Path,
        name: str,
        skill_type: SkillType,
        substitutions: Dict[str, Any]
    ) -> Path:
        test_dir = skill_dir / "tests"
        test_dir.mkdir(parents=True, exist_ok=True)
        file_name = substitutions.get("file_name", self._to_file_name(name))
        class_name = self._to_class_name(name)
        test_content = f'''import unittest
import sys
import os

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))


class Test{class_name}(unittest.TestCase):
    """RED Phase: Tests written before implementation."""

    def test_skill_import(self):
        """Should be able to import the skill module."""
        try:
            from ..{file_name} import {class_name}
            self.assertTrue(True, "Import successful")
        except ImportError as e:
            self.fail(f"Failed to import: {{e}}")

    def test_skill_instantiation(self):
        """Should be able to instantiate the skill class."""
        from ..{file_name} import {class_name}
        skill = {class_name}()
        self.assertIsNotNone(skill)

    def test_basic_functionality(self):
        """RED phase: Should FAIL until GREEN phase implementation."""
        from ..{file_name} import {class_name}
        skill = {class_name}()
        self.fail("Basic functionality test not yet implemented")


if __name__ == '__main__':
    unittest.main()
'''
        test_path = test_dir / f"test_{file_name}.py"
        with open(test_path, 'w', encoding='utf-8') as f:
            f.write(test_content)
        return test_path

    def _register_with_ecm(self, skill_dir: Path, name: str, skill_type: SkillType, substitutions: Dict[str, Any]) -> None:
        ecm_record = {
            "task": {
                "id": f"skill:{name}:create",
                "description": f"Create skill: {name}",
                "type": "skill_creation",
                "skill_type": skill_type.value,
                "status": "completed"
            },
            "attempt": {
                "id": f"attempt:{name}:001",
                "task_id": f"skill:{name}:create",
                "status": "completed",
                "timestamp": substitutions.get("timestamp", "")
            },
            "artifact_reference": {
                "path": str(skill_dir),
                "type": "skill",
                "name": name
            }
        }
        ecm_path = self.worktree_path / ".vibe" / "ecm-state.json"
        ecm_path.parent.mkdir(parents=True, exist_ok=True)
        if ecm_path.exists():
            with open(ecm_path, 'r', encoding='utf-8') as f:
                existing = json.load(f)
                existing.setdefault("skills", []).append(ecm_record)
        else:
            existing = {"skills": [ecm_record]}
        with open(ecm_path, 'w', encoding='utf-8') as f:
            json.dump(existing, f, indent=2)

    def _store_in_amx(self, skill_dir: Path, name: str, substitutions: Dict[str, Any]) -> None:
        amx_record = {
            "schema_version": "amx-memory-v1",
            "origin": substitutions.get("origin", "vibe:skill-creator"),
            "logical_identity": substitutions.get("logical_identity", f"{name}-v1"),
            "repository_scope": {
                "repository_revision": "unknown",
                "repository_path": str(skill_dir.parent),
                "worktree_id": "amcx1-vibe-integration"
            },
            "provenance": [{
                "source": "skill-creator",
                "timestamp": substitutions.get("timestamp", ""),
                "description": f"Skill '{name}' created"
            }],
            "causal_ancestry": [],
            "trust_validity_state": {
                "trust_level": "medium",
                "validity_status": "valid",
                "last_validated": substitutions.get("timestamp", "")
            },
            "visibility": {"scope": "workspace", "access_level": "read"},
            "purpose": {
                "primary": substitutions.get("primary_purpose", "skill:definition"),
                "task_class": substitutions.get("task_class", "memory:amx")
            },
            "retraction_deletion_barriers": {
                "can_retract": False,
                "can_delete": False,
                "retention_period": "infinite"
            },
            "canonical_semantic_digest": self._compute_digest(substitutions)
        }
        amx_path = self.worktree_path / ".vibe" / "amcx-memory" / "state.json"
        amx_path.parent.mkdir(parents=True, exist_ok=True)
        if amx_path.exists():
            with open(amx_path, 'r', encoding='utf-8') as f:
                existing = json.load(f)
                existing.setdefault("memories", []).append(amx_record)
        else:
            existing = {"memories": [amx_record]}
        with open(amx_path, 'w', encoding='utf-8') as f:
            json.dump(existing, f, indent=2)

    def _compute_digest(self, data: Dict[str, Any]) -> str:
        data_str = json.dumps(data, sort_keys=True)
        return hashlib.sha256(data_str.encode('utf-8')).hexdigest()

    def validate_skill(
        self,
        skill_path: Union[str, Path],
        implementation_path: Optional[Union[str, Path]] = None,
        level: int = 4
    ) -> ValidationResult:
        skill_path = Path(skill_path)
        result = ValidationResult(
            valid=True,
            level=level,
            errors=[],
            warnings=[],
            info=[],
            compliance={}
        )
        if level >= ValidationLevel.BASIC.value:
            self._validate_basic_structure(skill_path, result)
        if level >= ValidationLevel.AGENTS_MD.value:
            impl_path = Path(implementation_path) if implementation_path else None
            if not impl_path:
                skill_dir = skill_path.parent
                for candidate in ["skill.py", f"{self._to_file_name(skill_path.stem)}.py"]:
                    test_path = skill_dir / candidate
                    if test_path.exists():
                        impl_path = test_path
                        break
            if impl_path:
                self._validate_agents_md_compliance(impl_path, result)
        if level >= ValidationLevel.AMCX1.value:
            self._validate_amcx1_compliance(skill_path, result)
        result.valid = len(result.errors) == 0
        return result

    def _validate_basic_structure(self, skill_path: Path, result: ValidationResult) -> None:
        if not skill_path.exists():
            result.errors.append(f"Not found: {skill_path}")
            return
        with open(skill_path, 'r', encoding='utf-8') as f:
            content = f.read()
        if not content.strip().startswith('---'):
            result.errors.append("SKILL.md must start with frontmatter (---)")
        required_fields = ["name", "version", "description"]
        for field in required_fields:
            if f"{field}:" not in content:
                result.errors.append(f"Missing frontmatter field: {field}")
        if "model:" not in content:
            result.warnings.append("Missing 'model' field")
        if "preferred_tools:" not in content:
            result.warnings.append("Missing 'preferred_tools' field")
        if "temperature:" not in content:
            result.warnings.append("Missing 'temperature' field")

    def _validate_agents_md_compliance(self, impl_path: Path, result: ValidationResult) -> None:
        with open(impl_path, 'r', encoding='utf-8') as f:
            content = f.read()
        for rule_name, rule in AGENTS_MD_RULES.items():
            if rule_name == "cline_fabric_purity":
                for banned in rule.get("banned", []):
                    if banned in content:
                        result.errors.append(f"Violates {rule_name}: '{banned}' found")
            if rule_name == "tool_permissions":
                for tool in rule.get("allowed_tools", []):
                    if f"{tool}(" in content or f"'{tool}'" in content:
                        result.info.append(f"Uses allowed tool: {tool}")
        if "../" in content or "/etc/" in content or "/root/" in content:
            result.warnings.append("Potential workspace confinement issue")

    def _validate_amcx1_compliance(self, skill_path: Path, result: ValidationResult) -> None:
        if not skill_path.exists():
            return
        with open(skill_path, 'r', encoding='utf-8') as f:
            content = f.read()
        frontmatter = self._extract_frontmatter(content)
        if not frontmatter:
            result.warnings.append("Cannot parse frontmatter")
            return
        skill_type = frontmatter.get("skill_type", "")
        if "amx" in skill_type.lower() or "memory" in skill_type.lower():
            for field in AMX_MEMORY_REQUIRED_FIELDS:
                if field not in content:
                    result.warnings.append(f"AMX skill should reference: {field}")
        if "ecm" in skill_type.lower():
            for entity in ECM_REQUIRED_ENTITIES:
                if entity.lower() not in content.lower():
                    result.info.append(f"ECM skill could reference: {entity}")
        if "evidence" in skill_type.lower():
            if "verification" not in content.lower():
                result.info.append("Evidence skills should reference verification")

    def _extract_frontmatter(self, content: str) -> Dict[str, str]:
        if not content.strip().startswith('---'):
            return {}
        lines = content.split('\n')
        frontmatter_lines = []
        in_frontmatter = False
        for line in lines:
            if line.strip() == '---':
                if in_frontmatter:
                    break
                in_frontmatter = True
                continue
            if in_frontmatter:
                frontmatter_lines.append(line)
        if not frontmatter_lines:
            return {}
        frontmatter = {}
        for line in frontmatter_lines:
            line = line.strip()
            if ':' in line and not line.startswith('#'):
                key, value = line.split(':', 1)
                frontmatter[key.strip()] = value.strip().strip('"\'')
        return frontmatter

    def get_skill_metadata(self, skill_path: Union[str, Path]) -> SkillMetadata:
        skill_path = Path(skill_path)
        metadata = SkillMetadata(name="unknown")
        if skill_path.exists():
            with open(skill_path, 'r', encoding='utf-8') as f:
                content = f.read()
            frontmatter = self._extract_frontmatter(content)
            if frontmatter:
                metadata.name = frontmatter.get("name", metadata.name)
                metadata.version = frontmatter.get("version", metadata.version)
                metadata.description = frontmatter.get("description", metadata.description)
                metadata.model = frontmatter.get("model", metadata.model)
                metadata.temperature = float(frontmatter.get("temperature", metadata.temperature))
                tools_str = frontmatter.get("preferred_tools", "[]")
                try:
                    metadata.preferred_tools = json.loads(tools_str)
                except json.JSONDecodeError:
                    metadata.preferred_tools = [t.strip() for t in tools_str.strip('[]').split(',')]
        return metadata

    def check_compliance_status(
        self,
        skill_path: Union[str, Path],
        implementation_path: Optional[Union[str, Path]] = None
    ) -> Dict[str, ComplianceStatus]:
        result = self.validate_skill(skill_path, implementation_path, level=4)
        return result.compliance

    def generate_compliance_report(
        self,
        skill_path: Union[str, Path],
        implementation_path: Optional[Union[str, Path]] = None
    ) -> str:
        result = self.validate_skill(skill_path, implementation_path, level=4)
        lines = [
            "=" * 70,
            "SKILL COMPLIANCE REPORT",
            "=" * 70,
            "",
            f"Validation Level: {result.level}",
            f"Overall Status: {'PASS' if result.valid else 'FAIL'}",
            "",
        ]
        if result.errors:
            lines.append("ERRORS:")
            lines.append("-" * 70)
            for error in result.errors:
                lines.append(f"  - {error}")
            lines.append("")
        if result.warnings:
            lines.append("WARNINGS:")
            lines.append("-" * 70)
            for warning in result.warnings:
                lines.append(f"  - {warning}")
            lines.append("")
        if result.info:
            lines.append("INFO:")
            lines.append("-" * 70)
            for info in result.info:
                lines.append(f"  - {info}")
            lines.append("")
        lines.append("=" * 70)
        return "\n".join(lines)


def create_skill(
    name: str,
    skill_type: Union[str, SkillType] = SkillType.GENERAL,
    description: str = "",
    output_path: Optional[str] = None,
    **kwargs
) -> Path:
    creator = SkillCreator()
    return creator.create_skill(
        name=name,
        skill_type=skill_type,
        description=description,
        output_path=output_path,
        **kwargs
    )


def validate_skill(
    skill_path: Union[str, Path],
    implementation_path: Optional[Union[str, Path]] = None,
    level: int = 4
) -> ValidationResult:
    creator = SkillCreator()
    return creator.validate_skill(skill_path, implementation_path, level)


def list_templates() -> List[Dict[str, Any]]:
    creator = SkillCreator()
    return creator.list_templates()


def generate_compliance_report(
    skill_path: Union[str, Path],
    implementation_path: Optional[Union[str, Path]] = None
) -> str:
    creator = SkillCreator()
    return creator.generate_compliance_report(skill_path, implementation_path)


def self_prompt():
    print("=" * 70)
    print("SKILL CREATOR - Self-Prompt Mode")
    print("=" * 70)
    print()
    print("This meta-skill creates agentic skills complying with:")
    print("  - AutoDev AGENTS.md boundaries")
    print("  - AMCX-1 v1.1 AMX/ECM requirements")
    print("  - Vibe tool permission model")
    print("  - Cline Skills/Hooks/Plugins architecture")
    print()
    creator = SkillCreator()
    templates = creator.list_templates()
    print("Available Skill Types:")
    print("-" * 70)
    for template in templates:
        print(f"  {template['name']:15} - {template['description']}")
    print()
    print("Usage: from skills.skill_creator import SkillCreator")
    print("       creator = SkillCreator()")
    print("       creator.create_skill(name='my-skill', skill_type='amx-memory')")
    print("=" * 70)


if __name__ == "__main__":
    self_prompt()


__all__ = [
    "SkillCreator",
    "SkillValidationError",
    "SkillCreationError",
    "SkillType",
    "ValidationLevel",
    "ComplianceStatus",
    "ValidationResult",
    "SkillMetadata",
    "create_skill",
    "validate_skill",
    "list_templates",
    "generate_compliance_report",
    "self_prompt",
]
