import copy
import json
import os
import hashlib
from typing import Dict, Any, Optional

class ContractRegistryError(Exception):
    pass

class NeutralContractRegistry:
    """
    Git-backed Neutral Contract Registry.
    Guarantees that schema publication and activation are exclusively governed by
    the reviewed repository state (ADR-002, ADR-004 Domain #16).
    """
    def __init__(self, registry_dir: str):
        self.registry_dir = registry_dir
        self.v1_dir = os.path.join(registry_dir, "v1")
        self.manifest_file = os.path.join(registry_dir, "manifest.json")
        self._schemas: Dict[str, Dict[str, Any]] = {}
        self._manifest: Dict[str, Any] = {}
        self.load_registry()

    def load_registry(self) -> None:
        if not os.path.exists(self.manifest_file):
            raise ContractRegistryError(f"Registry manifest not found at {self.manifest_file}")
        
        with open(self.manifest_file, "r") as f:
            self._manifest = json.load(f)
            
        for schema_name, meta in self._manifest.get("schemas", {}).items():
            schema_path = os.path.join(self.v1_dir, schema_name)
            if not os.path.exists(schema_path):
                raise ContractRegistryError(f"Declared schema file {schema_name} missing on disk.")
            
            with open(schema_path, "rb") as f:
                data = f.read()
                computed_digest = hashlib.sha256(data).hexdigest()
                if computed_digest != meta["sha256"]:
                    raise ContractRegistryError(
                        f"Integrity violation on {schema_name}: expected {meta['sha256']}, got {computed_digest}"
                    )
                self._schemas[schema_name] = json.loads(data.decode("utf-8"))

    def get_schema(self, schema_name: str) -> Dict[str, Any]:
        if schema_name not in self._schemas:
            raise ContractRegistryError(f"Unknown or unactivated schema: {schema_name}. Fail-closed.")
        return copy.deepcopy(self._schemas[schema_name])

    def list_active_schemas(self) -> Dict[str, str]:
        return {k: v["sha256"] for k, v in self._manifest.get("schemas", {}).items()}

    def validate_schema_activation_authority(self, requester_role: str) -> bool:
        # Only repository maintainers / ADR process can activate schemas; runtime agents FAIL CLOSED
        if requester_role in ["MAINTAINER_ADR_REVIEW", "REPOSITORY_STEWARD"]:
            return True
        return False
