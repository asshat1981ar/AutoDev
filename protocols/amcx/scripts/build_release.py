#!/usr/bin/env python3
import argparse, hashlib, os, shutil, subprocess, sys, tempfile, zipfile
from pathlib import Path

EXCLUDED_DIRS={".git",".worktrees",".autodev",".superpowers","__pycache__","dist"}
def include(rel):
    if any(part in EXCLUDED_DIRS for part in rel.parts): return False
    if rel.suffix in {".pyc",".pyo"}: return False
    return True

def main():
    ap=argparse.ArgumentParser()
    ap.add_argument("--source",required=True)
    ap.add_argument("--output",required=True)
    a=ap.parse_args()
    src=Path(a.source).resolve(); out=Path(a.output).resolve()
    out.parent.mkdir(parents=True,exist_ok=True)
    with zipfile.ZipFile(out,"w",compression=zipfile.ZIP_DEFLATED) as z:
        for p in sorted(src.rglob("*")):
            if p.is_file():
                rel=p.relative_to(src)
                if include(rel) and p.resolve()!=out:
                    info=zipfile.ZipInfo(rel.as_posix(), date_time=(1980,1,1,0,0,0))
                    info.external_attr=(0o755 if os.access(p,os.X_OK) else 0o644)<<16
                    z.writestr(info,p.read_bytes(),compress_type=zipfile.ZIP_DEFLATED)
    with tempfile.TemporaryDirectory() as td:
        with zipfile.ZipFile(out) as z: z.extractall(td)
        runner=Path(td)/"scripts"/"run_verification.sh"
        p=subprocess.run(["bash",str(runner)],cwd=td)
        if p.returncode: return p.returncode
    digest=hashlib.sha256(out.read_bytes()).hexdigest()
    Path(str(out)+".sha256").write_text(f"{digest}  {out.name}\n")
    print(digest)
    return 0
if __name__=="__main__":
    raise SystemExit(main())
