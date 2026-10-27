"""Review checks. No installation, OS effects or host/SDK working-tree build."""
import pathlib,subprocess,os,json
v=pathlib.Path(__file__).resolve().parents[1];repo=v.parent;q=v/'.qa';q.mkdir(exist_ok=True)
for name in ['home','tmp','npm-cache']:(q/name).mkdir(exist_ok=True)
e=dict(os.environ,RUSTUP_HOME=os.environ.get('RUSTUP_HOME',str(pathlib.Path.home()/'.rustup')),HOME=str(q/'home'),TMPDIR=str(q/'tmp'),CARGO_HOME=str(q/'cargo-home'),npm_config_cache=str(q/'npm-cache'))
steps=[('fmt',['cargo','+stable','fmt','--manifest-path',str(v/'Cargo.toml'),'--','--check']),('stable-tests',['cargo','+stable','test','--manifest-path',str(v/'Cargo.toml'),'--locked','--all-targets']),('msrv-tests',['cargo','+1.85.0','test','--manifest-path',str(v/'Cargo.toml'),'--locked','--all-targets']),('stable-clippy',['cargo','+stable','clippy','--manifest-path',str(v/'Cargo.toml'),'--locked','--all-targets','--','-D','warnings']),('msrv-clippy',['cargo','+1.85.0','clippy','--manifest-path',str(v/'Cargo.toml'),'--locked','--all-targets','--','-D','warnings']),('production-bins',['cargo','+stable','build','--manifest-path',str(v/'Cargo.toml'),'--locked','--bins']),('domain-oracle',['python3',str(v/'qa/compare.py')]),('actual-renderer',['node',str(v/'qa/renderer-tests.cjs')]),('actual-worker-copy',['node',str(v/'qa/worker-copy-tests.cjs')]),('actual-schema',['node',str(v/'qa/validate-schema.cjs')]),('production-sdk-process',['python3',str(v/'qa/process-tests.py')])]
report={}
for name,cmd in steps:
 r=subprocess.run(cmd,cwd=repo,env=e,capture_output=True,text=True);out=r.stdout+r.stderr;(q/(name+'.log')).write_text(out);report[name]={'command':cmd,'exit':r.returncode,'output':out};print(name,r.returncode,flush=True)
 (v/'evidence/validation.json').write_text(json.dumps(report,indent=2)+'\n')
 if r.returncode:raise SystemExit(out)
print('All review checks passed')
