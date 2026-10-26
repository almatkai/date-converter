"""Exact whole-source parity, with a closed list of case-specific expected results."""
import pathlib,json,subprocess
v2=pathlib.Path(__file__).resolve().parents[1]
tests=json.loads((v2/'evidence/source-vectors.json').read_text())
exclusions=json.loads((v2/'qa/expected-exclusions.json').read_text())
assert len(tests)==2239 and len(exclusions)==31
by_index={x['index']:x for x in exclusions};assert len(by_index)==31
actual=json.loads(subprocess.run([str(v2/'target/debug/domain-oracle')],input=json.dumps(tests),text=True,capture_output=True,check=True).stdout)
assert len(actual)==len(tests), 'oracle output truncated'
fail=[];limits=[];passed=0
for i,(t,a) in enumerate(zip(tests,actual)):
 if i in by_index:
  x=by_index[i]
  if x['case']!=t or x['native']!=a or t['expected']==a:fail.append(dict(index=i,test=t,actual=a,expectedExclusion=x))
  else:limits.append(dict(index=i,text=t['text'],tz=t['tz'],target=t.get('target'),native=a,source=t['expected'],reason=x['reason']))
 elif t['expected']==a:passed+=1
 else:fail.append(dict(index=i,test=t,actual=a))
(v2/'.qa/domain-differences.json').write_text(json.dumps(fail,indent=2))
(v2/'evidence/domain-checks.json').write_text(json.dumps(dict(total=len(tests),exact=passed,explicitLimits=limits,unresolved=fail),indent=2)+'\n')
print(len(tests),'vectors;',passed,'exact;',len(limits),'explicit limits;',len(fail),'unresolved')
assert not fail, json.dumps(fail[:2],indent=2)
assert (passed,len(limits))==(2208,31)
