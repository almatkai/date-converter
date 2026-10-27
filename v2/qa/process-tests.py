"""Real JSONL pipes, actual SDK runner and production executable. No OS effects."""
import subprocess,pathlib,json,uuid,selectors,time,sys,os
v2=pathlib.Path(__file__).resolve().parents[1]
ID='com.tezbar.native.date-converter'
CAPS=['clipboard.write','clipboard.pasteText','settings.open']
PRODUCERS=json.loads((v2/'.qa/host-producers.json').read_text())
class Host:
 def __init__(self,binary=None):
  self.root=v2/'.qa/process'/uuid.uuid4().hex;self.root.mkdir(parents=True)
  env={'PATH':os.environ['PATH'],'TZ':'UTC','HOME':str(self.root),'TMPDIR':str(self.root)}
  self.p=subprocess.Popen([str(binary or os.environ.get('QA_BINARY') or v2/'target/debug/date-converter')],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE,env=env,text=True,bufsize=1)
  self.selector=selectors.DefaultSelector();self.selector.register(self.p.stdout,selectors.EVENT_READ);self.frames=[];self.buffer=b'';self.calls=[];self.readreply='2024-11-01T01:56:27Z';self.ack=None
  hello=self.read();assert hello['type']=='hello' and hello['id']==ID and hello['protocol']==1 and hello['sdkVersion']=='0.1.1',hello
  r=self.request('initialize',dict(protocol=1,extensionId=ID,platform='macOS',packageDir=str(v2),dataDir=str(self.root/'data'),tempDir=str(self.root/'tmp'),grantedCapabilities=CAPS,migrationPaths={}))
  assert r['result']['ready'] is True
 def send(self,v):self.p.stdin.write(json.dumps(v)+'\n');self.p.stdin.flush()
 def read(self):
  while b'\n' not in self.buffer:
   if not self.selector.select(5):raise AssertionError('JSONL timeout')
   chunk=os.read(self.p.stdout.fileno(),65536);assert chunk,'EOF '+self.p.stderr.read();self.buffer+=chunk
  line,self.buffer=self.buffer.split(b'\n',1);v=json.loads(line);self.frames.append(v);return v
 def begin(self,m,p):
  id='h:'+str(uuid.uuid4());self.send(dict(type='request',id=id,method=m,params=p));return id
 def reply(self,f,value=None,error=None):
  r=dict(type='response',id=f['id']);r['error' if error else 'result']=error if error else value;self.send(r)
 def wait(self,id):
  while True:
   f=self.read()
   if f['type']=='request':
    self.calls.append(f)
    if f['method']=='host.clipboard.read':self.reply(f,self.readreply)
    else:
     assert f['method'] in CAPS_HOST,f
     self.reply(f,PRODUCERS[f['method']] if self.ack is None else self.ack)
   elif f['type']=='response' and f['id']==id:return f
 def request(self,m,p):return self.wait(self.begin(m,p))
 def open(self,s='s',cmd='index',args=None,prefs=None,search=None):
  p=dict(sessionId=s,commandName=cmd,argumentValues=args or {},preferences=prefs or {})
  if search is not None:p['searchText']=search
  return self.request('session.open',p)
 def call(self,s,op,payload=None):return self.request('session.call',dict(sessionId=s,operation='extension:'+op,payload=payload or {}))
 def action(self,view,index=0,fields=None):return {'actionId':view['actions'][index]['id'],'formValues':fields if fields is not None else self.fields(view)}
 def fields(self,view):return dict(__viewToken=view['root']['props']['__draftGuard'],__draftDirty=False,__searchDirty=False,__accessoryValue=view['root']['props']['__timezone'],__searchText=view['root']['props']['searchText'])
 def dispose(self,s='s'):assert 'result'in self.request('session.dispose',{'sessionId':s})
 def finish(self):
  self.p.stdin.close();self.p.wait(timeout=5);assert self.p.returncode==0,self.p.stderr.read();self.selector.close()
CAPS_HOST=['host.clipboard.write','host.clipboard.pasteText','host.settings.open','host.window.visibility']
checks=[]
def check(name,fn):fn();checks.append(name)
def row_actions(h,v,row=0):
 ids=v['root']['children'][row]['props'].get('actionIds',[]);return [next(a for a in v['actions'] if a['id']==i) for i in ids]
def invoke(h,v,s='s',row=0,fmt=0,fields=None):
 a=row_actions(h,v,row)[fmt];return h.call(s,'invoke-action',{'actionId':a['id'],'formValues':fields if fields is not None else h.fields(v)})
def zone(h,v,value,s='s'):
 return h.call(s,'invoke-action',{'actionId':v['root']['props']['searchBarAccessory']['props']['actionId'],'formValues':dict(h.fields(v),value=value,__draftDirty=True,__accessoryValue=value)})
def workflow():
 h=Host();v=h.open()['result'];assert len(v['root']['children'])==4 and not h.calls
 v=h.call('s','search-text-changed',{'searchText':'2024-11-01T01:56:27Z'})['result'];old=h.action(v)
 h.call('s','search-text-changed',{'searchText':'tomorrow'});a=h.call('s','search-text-changed',{'searchText':'2024-11-01T01:56:27Z'})['result'];assert 'error'in h.call('s','invoke-action',old)
 for fields in [{},dict(h.fields(a),__draftDirty=True),dict(h.fields(a),__searchText='B'),dict(h.fields(a),__accessoryValue='Asia/Tokyo'),dict(h.fields(a),__searchDirty=True)]:assert 'error'in invoke(h,a,fields=fields)
 r=invoke(h,a)['result'];assert r['mode']=='view';assert [f['method'] for f in h.calls]==['host.clipboard.write','host.clipboard.pasteText'];assert all(f['params']=={'text':'1730426187000'} for f in h.calls)
 assert 'error'in h.call('s','invoke-action',h.action(a));h.dispose();h.finish()
check('empty defaults, current draft, search A-B-A, exact copy/both actions, consumed IDs',workflow)
def formats():
 h=Host()
 for i,mode in enumerate(['copy','paste','both']):
  v=h.open(str(i),prefs={'defaultFormat':'iso','secondaryFormat':'human','copyAction':mode,'humanFormat':'YYYY/MM/DD HH:mm'},search='2024-01-15T12:00:00Z')['result'];assert [a['title'] for a in row_actions(h,v)]==['Copy as ISO Date','Copy as Human Date','Copy as Unix Timestamp (ms)','Copy as Unix Timestamp (seconds)'];start=len(h.calls);r=invoke(h,v,str(i),fmt=1)['result'];assert r['mode']=='view';assert [f['method'] for f in h.calls[start:]]=={'copy':['host.clipboard.write'],'paste':['host.clipboard.pasteText'],'both':['host.clipboard.write','host.clipboard.pasteText']}[mode];assert h.calls[-1]['params']=={'text':'2024/01/15 12:00'};h.dispose(str(i))
 h.finish()
check('all four formats, preferences, copy/paste/both order and no additional hide',formats)
def editing():
 h=Host();v=h.open(search='<img src=x onerror=alert(1)>')['result'];assert v['root']['props']['searchText'].startswith('<img') and v['root']['children'][0]['type']=='List.EmptyView'
 v=h.call('s','search-text-changed',{'searchText':'tomorrow and next Friday'})['result'];assert len(v['root']['children'])==2
 v=h.call('s','search-text-changed',{'searchText':'bad date'})['result'];assert v['root']['children'][0]['props']['title']=='Invalid Date';assert not any(a['title'].startswith('Copy as') for a in v['actions'])
 v=h.call('s','search-text-changed',{'searchText':'2024-01-15T12:00:00Z'})['result'];assert len(v['root']['children'])==1;h.dispose();h.finish()
check('multi-result parsing, invalid-input editing, no old output copy',editing)
def bad_acks():
 h=Host()
 for i,ack in enumerate([True,None,{}, {'ok':False},{'ok':True,'error':'contradiction'},{'ok':True,'message':None},{'ok':True,'extra':True}]):
  v=h.open(str(i),prefs={'copyAction':'paste'},search='2024-11-01T01:56:27Z')['result'];p=h.action(v)
  id=h.begin('session.call',dict(sessionId=str(i),operation='extension:invoke-action',payload=p));f=h.read();assert f['method']=='host.clipboard.pasteText';prepared=f['params']['text'];h.reply(f,ack);r=h.wait(id)['result'];assert r['mode']=='view' and 'confirm'in r['message'];assert 'May Repeat'in r['actions'][0]['title'];assert 'error'in h.call(str(i),'invoke-action',p)
  # An edited search must not reparse a prepared delivery.
  r=h.call(str(i),'search-text-changed',{'searchText':'now'})['result'];r2=h.call(str(i),'invoke-action',h.action(r))['result'];assert r2['mode']=='view' and h.calls[-1]['params']['text']==prepared;h.dispose(str(i))
 h.finish()
check('closed ACKs reject null/bool/contradictions; explicit retry preserves prepared bytes',bad_acks)
def cancel_effect():
 h=Host();v=h.open(prefs={'copyAction':'paste'},search='2024-11-01T01:56:27Z')['result'];old=h.action(v);id=h.begin('session.call',dict(sessionId='s',operation='extension:invoke-action',payload=old));effect=h.read();prepared=effect['params'];h.send({'type':'cancel','requestId':id,'reason':'synthetic cancellation'});assert 'error'in h.wait(id);h.reply(effect,PRODUCERS['host.clipboard.pasteText']);v=h.call('s','refresh-session')['result'];r=h.call('s','invoke-action',h.action(v));assert r['result']['mode']=='view' and h.calls[-1]['params']==prepared;assert 'error'in h.call('s','invoke-action',old);h.dispose();h.finish()
check('canceled effect, late ACK and stale action cannot automatically replay',cancel_effect)
def checkpoint():
 h=Host();v=h.open(search='2024-11-01T01:56:27Z')['result'];id=h.begin('session.call',dict(sessionId='s',operation='extension:invoke-action',payload=h.action(v)))
 f=h.read();assert f['method']=='host.clipboard.write';h.reply(f,PRODUCERS['host.clipboard.write']);f=h.read();assert f['method']=='host.clipboard.pasteText';text=f['params'];h.reply(f,False);v=h.wait(id)['result'];start=len(h.calls);h.call('s','invoke-action',h.action(v));assert [f['method']for f in h.calls[start:]]==['host.clipboard.pasteText'] and h.calls[-1]['params']==text;h.dispose();h.finish()
check('both mode confirms write stage; uncertain paste retries paste only',checkpoint)
def canceled_opens():
 h=Host();canceled=0;completed=0
 for i in range(140):
  s='c'+str(i);id='h:'+str(uuid.uuid4());h.p.stdin.write(json.dumps(dict(type='request',id=id,method='session.open',params=dict(sessionId=s,commandName='index',preferences={},argumentValues={})))+'\n'+json.dumps(dict(type='cancel',requestId=id,reason='synthetic canceled open'))+'\n');h.p.stdin.flush();r=h.wait(id)
  if 'result'in r:completed+=1;h.dispose(s)
  else:canceled+=1
 assert canceled==140,(canceled,completed)
 global canceled_open_counts;canceled_open_counts={'canceled':canceled,'completedBeforeCancel':completed}
 v=h.open('final');assert v['result']['mode']=='view';h.dispose('final');h.finish()
check('140 canceled opens: SDK-owned scopes, adapter drop guard and bounded prune',canceled_opens)
def saturation():
 h=Host()
 for i in range(128):assert 'result'in h.open(str(i),search='garbage')
 assert 'error'in h.open('overflow')
 for i in range(128):h.dispose(str(i))
 assert 'result'in h.open('recovered');h.dispose('recovered');h.finish()
check('128-session capacity and recovery after dispose',saturation)
def eof_pending():
 h=Host();v=h.open(prefs={'copyAction':'paste'},search='now')['result'];h.begin('session.call',dict(sessionId='s',operation='extension:invoke-action',payload=h.action(v)));f=h.read();assert f['method']=='host.clipboard.pasteText';h.finish()
check('EOF cancels reverse RPC and terminates production process',eof_pending)
def persistence():
 h=Host();v=h.open(search='2024-01-15T12:00:00Z')['result'];v=zone(h,v,'Asia/Kathmandu')['result'];assert v['root']['props']['__timezone']=='Asia/Kathmandu';assert '05:45:00'in v['root']['children'][0]['props']['title'];store=h.root/'data/timezone/storage.json';p=json.loads(store.read_text());p['unknown']={'preserve':True};store.write_text(json.dumps(p));h.dispose();v=h.open()['result'];assert v['root']['props']['__timezone']=='Asia/Kathmandu';zone(h,v,'Europe/Berlin');assert json.loads(store.read_text())['unknown']=={'preserve':True};h.dispose();store.write_text('{corrupt');v=h.open()['result'];assert 'Corrupt'in v['message'];v=h.call('s','search-text-changed',{'searchText':'tomorrow'})['result'];assert v['root']['children'][0]['type']=='List.Item';assert store.read_text()=='{corrupt';h.dispose();h.finish()
 other=Host();v=other.open()['result'];assert v['root']['props']['__timezone']=='UTC';other.dispose();other.finish()
check('target dropdown persistence, unknown keys, profile isolation, corruption editable',persistence)
def timezone_recovery():
 h=Host();query='1870-01-01T00:00:00Z'
 v=h.open(search=query)['result'];original=v['root']['children'];old=h.action(v)
 for target in ['America/New_York','Europe/Berlin','Australia/Lord_Howe']:
  v=zone(h,v,target)['result'];assert 'subminute'in v['message'] and v['root']['children'][0]['type']=='List.EmptyView'
  assert not any(a['title'].startswith('Copy as')for a in v['actions'])
  assert json.loads((h.root/'data/timezone/storage.json').read_text())['value']==target
  assert 'error'in h.call('s','invoke-action',old)
  v=h.call('s','refresh-session')['result'];assert v['mode']=='view' and 'subminute'in v['message']
  v=zone(h,v,'UTC')['result'];assert 'message'not in v
  assert [r['props']['id']for r in v['root']['children']]==[r['props']['id']for r in original]
  assert [r['props']['title']for r in v['root']['children']]==[r['props']['title']for r in original]
 # Target civil overflow must also render a recoverable error, never stale state.
 h.dispose();v=h.open(search='9999-12-31T23:59:59Z')['result'];last_id=v['root']['children'][0]['props']['id']
 v=zone(h,v,'Europe/Berlin')['result'];assert '0100..9999'in v['message'] and not any(a['title'].startswith('Copy as')for a in v['actions'])
 v=zone(h,v,'UTC')['result'];assert v['root']['children'][0]['props']['id']==last_id
 h.dispose();v=h.open(search=query)['result']
 # Persisted valid IANA zone unsupported for this date; open must retain parsed results.
 v=zone(h,v,'America/New_York')['result'];h.dispose()
 v=h.open(search=query)['result'];assert 'subminute'in v['message']
 v=zone(h,v,'UTC')['result'];assert v['root']['children'][0]['props']['id']==original[0]['props']['id']
 # No partial rows when a later date fails formatting.
 h.dispose();v=h.open(search='January 15 2024 and January 1 1870')['result'];original_ids=[r['props']['id']for r in v['root']['children']];assert len(original_ids)==2
 v=zone(h,v,'America/New_York')['result'];assert len(v['root']['children'])==1 and v['root']['children'][0]['type']=='List.EmptyView';assert not any(a['title'].startswith('Copy as')for a in v['actions'])
 v=zone(h,v,'UTC')['result'];assert [r['props']['id']for r in v['root']['children']]==original_ids
 # Unsupported stored identifier is explicit, with no fallback copy actions.
 h.dispose();store=h.root/'data/timezone/storage.json';store.write_text(json.dumps({'value':'Unsupported/Zone'}))
 v=h.open(search=query)['result'];assert 'unsupported'in v['message'];assert not any(a['title'].startswith('Copy as')for a in v['actions'])
 v=h.call('s','search-text-changed',{'searchText':query})['result'];assert 'unsupported'in v['message'] and not any(a['title'].startswith('Copy as')for a in v['actions'])
 v=zone(h,v,'UTC')['result'];assert v['root']['children'][0]['props']['id']==original[0]['props']['id'];assert json.loads(store.read_text())['value']=='UTC'
 # Empty-search Now is memoized across repeated successful target toggles.
 h.dispose();v=h.open()['result'];memo=[r['props']['id']for r in v['root']['children']]
 for target in ['America/New_York','Europe/Berlin','Australia/Lord_Howe','UTC']*3:
  old=h.action(v);v=zone(h,v,target)['result'];assert [r['props']['id']for r in v['root']['children']]==memo;assert 'error'in h.call('s','invoke-action',old)
 assert not h.calls;h.dispose();h.finish()
check('timezone errors/refresh/persisted invalid target recover memoized dates atomically; repeated toggles reject stale actions without effects',timezone_recovery)
(v2/'evidence'/os.environ.get('QA_REPORT','process-checks.json')).write_text(json.dumps({'checks':checks,'canceledOpens':canceled_open_counts,'producerFixture':'tests/host_extracts.rs','productionExecutable':str(os.environ.get('QA_BINARY')or v2/'target/debug/date-converter')},indent=2)+'\n')
print(len(checks),'real SDK process scenarios passed')
