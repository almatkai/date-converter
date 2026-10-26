"""Reproduce locked V1 QA install inside ignored clone-local directories."""
import pathlib,subprocess,os,json
v2=pathlib.Path(__file__).resolve().parents[1];repo=v2.parent;pin='ad4ebc6856db6a9c554a4e4a093a319b12f66fdf';dest=v2/'.qa/v1';dest.mkdir(parents=True,exist_ok=True)
paths=subprocess.check_output(['git','-C',str(repo),'ls-tree','-r','--name-only',pin],text=True).splitlines()
for f in paths:
 p=dest/f;p.parent.mkdir(parents=True,exist_ok=True);p.write_bytes(subprocess.check_output(['git','-C',str(repo),'show',pin+':'+f]))
for n in ['home','tmp','npm-cache']:(v2/'.qa'/n).mkdir(exist_ok=True)
e=dict(os.environ,HOME=str(v2/'.qa/home'),TMPDIR=str(v2/'.qa/tmp'),npm_config_cache=str(v2/'.qa/npm-cache'))
subprocess.run(['npm','ci','--ignore-scripts','--no-audit','--no-fund'],cwd=dest,env=e,check=True)
