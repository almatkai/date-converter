import json,pathlib,datetime
now=1730426187456
texts=['',' ','now','today','tomorrow','yesterday','tomorrow at 3pm','Friday','next Monday','last weekend','this weekday','midnight','noon','tonight','last night','in 2 days','in 1.5 days','2.5 hours ago','in 1 month','in 1.5 months','next year','this month','this week','2 weeks from now','2 days after tomorrow','tomorrow + 2 hours','March 4 to March 7','3pm to 4pm','tomorrow and next Friday','Meet on March 4 at 3pm and July 7 at noon','May 10 24','May 10 25','July 20 99','2024-01-15T12:00:00Z','2024-02-30T12:00:00Z','2024-03-10T02:30:00','2024-11-03T01:30:00','2024-01-15T24:00:00Z','2024-13-01T12:00:00Z','2024-01-15T12:00:60Z','2024-01-15T12:00:00.12345+05:45','0','-1','-1000000000','1000000','1000001','2147483648','2147483649','1730426187','1730426187456','1730426187456123','1730426187456123456','9999999999999999999','0x672423ab','0X672423AB','1730426187.9','1730426187abc','1e12','-1.5','garbage','overmorrow','<img src=x onerror=alert(1)>','  tomorrow  ','tomorrow at 3pm EST','2024-12-31','2024-01-01','Feb 29 2024','Feb 29 2023','3/4/24','4th of July','tomorrow 9:30pm','in two years','next quarter','in 1 year 2 months 3 days']
zones=['UTC','America/New_York','Europe/Berlin','Asia/Kathmandu','Australia/Lord_Howe','Pacific/Apia']
out=[]
for z in zones:
 for t in texts:out.append(dict(text=t,now=now,tz=z))
for z in zones:
 for n in [1710045000123,1730608200123,1706659200123,1709164800123,1735689599123]:
  for t in ['','tomorrow','yesterday','in 1 month','in 0.5 days','in -0.5 days','in 2 hours','noon','today at 3pm']:out.append(dict(text=t,now=n,tz=z))
for f in ['YYYY YY MMMM MMM MM M DD D dddd ddd dd d HH H hh h mm m ss s SSS Z ZZ A a','Q Do X x k kk w ww W WW GGGG gggg z zzz','[safe <img>] L LL LLL LLLL LT LTS','  ','[foo] YYYY [bar]']:
 for z in zones:out.append(dict(text='2024-01-15T12:00:00.123Z',now=now,tz=z,target='America/New_York',preferences={'displayFormat':f,'humanFormat':f}))
for primary in ['human','unix-ms','unix-s','iso','unknown']:
 for secondary in ['human','unix-ms','unix-s','iso']:
  for action in ['copy','paste','both']:out.append(dict(text='tomorrow',now=now,tz='UTC',preferences={'defaultFormat':primary,'secondaryFormat':secondary,'copyAction':action}))
for n in [0,1,29999,30000,89999,90000,3569999,3570000,84599999,84600000,561599999,561600000,2327615999,2327616000,30242159999,30242160000]:
 for sign in [-1,1]:out.append(dict(text=str(now+n*sign),now=now,tz='UTC'))
# Boundary and grammar corpus, including unsupported ranges recorded explicitly.
extra=['2024-13-01T12:00:00','2024-02-31T12:00:00','2024-01-32T12:00:00','2024-01-01T25:00:00','2024-01-01T12:60:00','2024-01-01T12:00:00.1','0001-01-01T00:00:00Z','0099-01-01T00:00:00Z','0100-01-01T00:00:00Z','0999-01-01T00:00:00Z','1000-01-01T00:00:00Z','9999-12-31T23:59:59Z','8640000000000000','8640000000000001','9999999999999999','0000000000000000','0000000000000000001','0xffffffffffffffff','+2147483648','2147483648.999','1000000.999','0x100000','0x100001','0x80000000','0x80000001','01/02/03','March 4 at noon','May 10 12','March 4 00','March 4 50','March 4 51','Jan 31 2025 + 1 month','in 1.5 years','in 2.5 weeks','in 0.5 months','in 1 day 2 months','in 2 months 1 day','July 4, 2025 at 3:45:27.123pm','2 days before July 4, 2025','next Friday at 9am','5pm on Wednesday','this afternoon','yesterday evening','12am','5:00 AM PST','3pm EDT','June 1 at 1pm CET','March 4-7 2025','3-4pm','2024-01-15T12:00:00Z and tomorrow','  \ufeffnow\ufeff  ','July 20 2500','2025-02-29T12:00:00Z']
for z in zones:
 for t in extra:out.append(dict(text=t,now=now,tz=z))
for t in ['2024-01-01T00:00:00Z','tomorrow','']:
 for target in ['UTC','Europe/Berlin','America/New_York','Asia/Kathmandu','Australia/Lord_Howe','Pacific/Apia','Pacific/Chatham','Etc/GMT+12']:
  out.append(dict(text=t,now=now,tz='America/New_York',target=target))
for tz in zones:
 for text in ['2024-03-10T06:30:00.123Z','2024-03-10T07:30:00.123Z','2024-11-03T05:30:00.123Z','2024-11-03T06:30:00.123Z','1990-01-01T12:00:00Z']:
  for target in ['UTC','America/New_York','Europe/Berlin','Pacific/Kiritimati','Asia/Kathmandu']:
   out.append(dict(text=text,now=now,tz=tz,target=target,preferences={'displayFormat':'YYYY-MM-DD HH:mm:ss.SSS Z ZZ','humanFormat':'YYYY-MM-DD HH:mm:ss.SSS Z ZZ'}))
pathlib.Path('v2/qa/vectors.json').write_text(json.dumps(out,indent=2)+'\n')
