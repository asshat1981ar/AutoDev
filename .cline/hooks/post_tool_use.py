#!/usr/bin/env python3
import json, sys, time
event = json.loads(sys.stdin.read() or "{}")
print(json.dumps({"telemetry": {"timestamp": int(time.time()), "tool": event.get("tool_name", "unknown"), "status": event.get("status", "completed")}}))