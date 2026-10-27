"""Independent source copy, HTTPS git SDK pin, separate Cargo home and target."""
import pathlib,os,subprocess,json,hashlib,tempfile
v2=pathlib.Path(__file__).resolve().parents[1];repo=v2.parent;dest=v2/'.qa/isolated';dest.mkdir(parents=True,exist_ok=True)
files=subprocess.check_output(['git','-C',str(repo),'ls-files','--cached','--others','--exclude-standard','-z','v2']).decode().split('\0')
for f in files:
 if not f:continue
 src=repo/f;rel=src.relative_to(v2);d=dest/rel;d.parent.mkdir(parents=True,exist_ok=True);d.write_bytes(src.read_bytes())
# Every run starts with a fresh independent Cargo home; no sibling/cache reuse.
cache=pathlib.Path(tempfile.mkdtemp(prefix='isolated-cargo-',dir=v2/'.qa'));fresh=True
e=dict(os.environ,RUSTUP_HOME=os.environ.get('RUSTUP_HOME',str(pathlib.Path.home()/'.rustup')),CARGO_HOME=str(cache),HOME=str(v2/'.qa/home'),TMPDIR=str(v2/'.qa/tmp'),CARGO_TARGET_DIR=str(dest/'target'))
r=subprocess.run(['cargo','+stable','build','--release','--locked','--bin','date-converter'],cwd=dest,env=e,text=True,capture_output=True);(v2/'.qa/isolated-build.log').write_text(r.stdout+r.stderr)
assert r.returncode==0,r.stderr
report={'exit':r.returncode,'freshCacheThisRun':fresh,'sdk':'https://github.com/almatkai/tezbar-extension-sdk-rust.git?rev=e955911f0bf5b558f9d3eb5183df0adc95c22af2','lockSha256':hashlib.sha256((dest/'Cargo.lock').read_bytes()).hexdigest(),'localPathDependency':False,'binarySha256':hashlib.sha256((dest/'target/release/date-converter').read_bytes()).hexdigest()}
(v2/'evidence/isolated-build.json').write_text(json.dumps(report,indent=2)+'\n');print('Isolated HTTPS SDK release build passed')
