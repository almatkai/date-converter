"""Append deterministic review-2 source/target, century/leap and recurrence probes."""
import pathlib,json,calendar,datetime
v=pathlib.Path(__file__).resolve().parents[1]
base=json.loads((v/'qa/vectors.json').read_text());assert len(base)==1328
now=1730426187456
zones=['UTC','Etc/GMT+5','America/New_York','Europe/Berlin','Australia/Lord_Howe']
texts=['1899-12-31T12:00:00Z','1900-01-01T12:00:00Z','2099-12-31T12:00:00Z','2100-01-01T12:00:00Z','2099-07-20T12:00:00Z','2100-07-20T12:00:00Z','2100-02-28T12:00:00Z','2100-02-29T12:00:00Z','2100-03-01T12:00:00Z','2400-02-29T12:00:00Z','2400-07-20T12:00:00Z','2500-01-20T12:00:00Z','2500-07-20T12:00:00Z','9999999999999999999','9999999999999999','July 20 2500','February 29 2400','January 1 2100','in 100 years']
fmt='YYYY YY MMMM MMM MM M DD D dddd ddd dd d HH H hh h mm m ss s SSS Z ZZ A a [L] L LL LLL LLLL LT LTS'
out=[]
def add(text,source,target,**kw):out.append(dict(text=text,tz=source,target=target,now=now,preferences={'displayFormat':fmt,'humanFormat':fmt},**kw))
for text in texts:
 for source in zones:
  for target in zones:add(text,source,target)
# Actual local gaps/folds; both ISO machine and chrono parsing are exercised.
def sunday(y,m,n):
 days=[d for d in range(1,calendar.monthrange(y,m)[1]+1)if datetime.date(y,m,d).weekday()==6]
 return days[n-1] if n>0 else days[-1]
for y in [2099,2100,2400,2500]:
 for source,m,n,h in [('America/New_York',3,2,2),('America/New_York',11,1,1),('Europe/Berlin',3,-1,2),('Europe/Berlin',10,-1,2),('Australia/Lord_Howe',4,1,1),('Australia/Lord_Howe',10,1,2)]:
  day=sunday(y,m,n)
  for minute in [15,45]:
   for target in zones:add(f'{y}-{m:02}-{day:02}T{h:02}:{minute:02}:00',source,target)
# UTC and fixed-offset full practical civil bounds, pre-epoch input and natural date.
for text in ['0100-07-20T12:00:00Z','0999-07-20T12:00:00Z','1870-01-01T12:00:00Z','9999-07-20T12:00:00Z','July 20 9999']:
 for source in zones[:2]:
  for target in zones[:2]:add(text,source,target)
for primary in ['human','unix-ms','unix-s','iso']:
 for secondary in ['human','unix-ms','unix-s','iso']:
  for mode in ['copy','paste','both']:
   for text in ['2100-01-01T12:00:00Z','2400-02-29T12:00:00Z','July 20 2500']:
    out.append(dict(text=text,tz='America/New_York',target='Australia/Lord_Howe',now=now,preferences={'defaultFormat':primary,'secondaryFormat':secondary,'copyAction':mode,'displayFormat':fmt,'humanFormat':fmt}))
# Data-version regressions: Moldova's post-2022 EU transition times and
# Baja California's corrected 1953/1961-1975 summer rules (IANA 2026a).
for zone, dates in [
 ('Europe/Chisinau',['2024-03-31T00:30:00Z','2024-03-31T01:30:00Z','2024-10-27T00:30:00Z','2024-10-27T01:30:00Z']),
 ('America/Tijuana',['1953-07-20T12:00:00Z','1965-07-20T12:00:00Z','1975-07-20T12:00:00Z','1976-07-20T12:00:00Z'])]:
 for text in dates:
  for source in ['UTC',zone]:
   for target in ['UTC',zone]:add(text,source,target)
assert len(out)==911,len(out)
(v/'qa/vectors.json').write_text(json.dumps(base+out,indent=2)+'\n')
print(len(out),'review-2 probes;',len(base+out),'total')
