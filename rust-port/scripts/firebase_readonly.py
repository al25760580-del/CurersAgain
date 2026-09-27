#!/usr/bin/env python3
"""Linux transport helper: Firebase anonymous auth + exact Firestore document GET.
No access to Steam profiles, no account updates, no Firestore writes or listing.
Temporary stdlib transport; the Rust scene polls it asynchronously.
"""
import sys,json,time,os,re,urllib.request,urllib.error,urllib.parse,hashlib,signal
from pathlib import Path
MAX=4*1024*1024
class NoRedirect(urllib.request.HTTPRedirectHandler):
 def redirect_request(self,*args,**kwargs):return None
HTTP=urllib.request.build_opener(NoRedirect())
REQUEST_ROOT=None
def request(url,method='GET',body=None,token=None,form=False):
 if not (url.startswith('https://firestore.googleapis.com/v1/projects/') and method=='GET' or url.startswith('https://identitytoolkit.googleapis.com/v1/accounts:signUp?key=') and method=='POST' or url.startswith('https://securetoken.googleapis.com/v1/token?key=') and method=='POST'):raise ValueError('REQUEST_BLOCKED')
 if method=='GET' and REQUEST_ROOT is not None:
  stamp=REQUEST_ROOT/'last-read-at'
  if stamp.exists() and time.time()<float(stamp.read_text())+5:raise ValueError('READ_THROTTLED')
  stamp.write_text(str(time.time()))
 headers={'Content-Type':'application/x-www-form-urlencoded' if form else 'application/json'}
 if token:headers['Authorization']='Bearer '+token
 data=(urllib.parse.urlencode(body) if form else json.dumps(body)).encode() if body is not None else None
 try:
  with HTTP.open(urllib.request.Request(url,data=data,method=method,headers=headers),timeout=12) as r:
   raw=r.read(MAX+1)
   if len(raw)>MAX:raise ValueError('RESPONSE_TOO_LARGE')
   return json.loads(raw)
 except urllib.error.HTTPError as e:raise ValueError('HTTP_'+str(e.code)) from None
 except (urllib.error.URLError,TimeoutError):raise ValueError('NETWORK_UNAVAILABLE') from None

def store(path,value):
 tmp=path.with_suffix('.tmp');tmp.write_text(json.dumps(value));tmp.chmod(0o600);tmp.replace(path)
def authenticate(root,cfg):
 path=root/'client-session.private.json'
 if path.exists():
  session=json.loads(path.read_text())
  if session['expires_at']>time.time()+60:return session['id_token']
  result=request('https://securetoken.googleapis.com/v1/token?key='+cfg['WebAPIKey'],'POST',{'grant_type':'refresh_token','refresh_token':session['refresh_token']},form=True)
  session={'id_token':result['id_token'],'refresh_token':result['refresh_token'],'expires_at':time.time()+int(result['expires_in'])}
 else:
  result=request('https://identitytoolkit.googleapis.com/v1/accounts:signUp?key='+cfg['WebAPIKey'],'POST',{'returnSecureToken':True})
  session={'id_token':result['idToken'],'refresh_token':result['refreshToken'],'expires_at':time.time()+int(result['expiresIn'])}
 store(path,session);return session['id_token']
def unpack(v):
 if 'mapValue' in v:return {k:unpack(x) for k,x in v['mapValue'].get('fields',{}).items()}
 if 'arrayValue' in v:return [unpack(x) for x in v['arrayValue'].get('values',[])]
 for k in ['stringValue','booleanValue']: 
  if k in v:return v[k]
 if 'integerValue' in v:return int(v['integerValue'])
 if 'doubleValue' in v:return float(v['doubleValue'])
 if 'nullValue' in v:return None
 raise ValueError('UNSUPPORTED_FIRESTORE_TYPE')
def records(document,stage,character,period):
 fields=document.get('fields',{});bucket=unpack(fields.get('daily' if period=='daily' else 'allTime',{'mapValue':{}}))
 raw=bucket.get(character,[])
 if not isinstance(raw,list) or len(raw)>1000:raise ValueError('INVALID_SCORE_ARRAY')
 rows=[]
 for r in raw:
  # Compact wire keys observed in the official document; equipment remains in raw cache.
  name=r.get('n','');score=r.get('s');duration=r.get('d');level=r.get('l')
  if not isinstance(name,str) or len(name)>64 or any(ord(x)<32 or ord(x)==127 for x in name):raise ValueError('INVALID_SCORE_NAME')
  if any(not isinstance(v,(int,float)) or isinstance(v,bool) or int(v)!=v or v<0 for v in [score,duration,level]):raise ValueError('INVALID_SCORE_NUMBER')
  if score>999999999999 or max(duration,level)>4294967295:raise ValueError('SCORE_NUMBER_OVERFLOW')
  rows.append({'username':name,'stage':stage,'character':character,'score':int(score),'duration_seconds':int(duration),'level':int(level),'daily':period=='daily','is_player':False})
 return rows

def main():
 global REQUEST_ROOT
 os.umask(0o077)
 config,stage,character,period=sys.argv[1:]
 cfgpath=Path(config);cfg=json.loads(cfgpath.read_text());root=cfgpath.parent;REQUEST_ROOT=root
 if period not in ['daily','allTime']:raise ValueError('UNSUPPORTED_PERIOD')
 if stage not in cfg['stages'] or character not in cfg['groups']:raise ValueError('UNSUPPORTED_FILTER')
 project=cfg['ProjectID'];doc=cfg['stages'][stage]+cfg['groups'][character]
 if not re.fullmatch('[a-z0-9-]+',project) or not re.fullmatch('sortedLeaderboards/[a-zA-Z0-9]+',doc):raise ValueError('INVALID_CONFIG')
 cache=root/'readonly-cache';cache.mkdir(mode=0o700,exist_ok=True)
 path=cache/(hashlib.sha256((project+'/'+doc).encode()).hexdigest()+'.json')
 cached=json.loads(path.read_text()) if path.exists() else None
 if cached and time.time()<=cached['fetched_at']+10800:
  document=cached['document']
 else:
  token=authenticate(root,cfg)
  document=request('https://firestore.googleapis.com/v1/projects/'+project+'/databases/(default)/documents/'+doc,token=token)
  store(path,{'fetched_at':time.time(),'document':document})
 print(json.dumps({'schema':1,'records':records(document,stage,character,period)}))
def deadline(signum,frame):raise ValueError('CLIENT_TIMEOUT')
if __name__=='__main__':
 signal.signal(signal.SIGALRM,deadline);signal.alarm(30)
 lock=None
 try:
  os.umask(0o077)
  candidate=Path(sys.argv[1]).parent/'client-request.lock'
  try:candidate.mkdir(mode=0o700)
  except FileExistsError:raise ValueError('CLIENT_BUSY')
  lock=candidate
  main()
 except Exception as e:
  # Never emit exception URLs, tokens, payloads, or response bodies.
  code=str(e) if isinstance(e,ValueError) and re.fullmatch('[A-Z_0-9]+',str(e)) else 'CLIENT_ERROR'
  print(code,file=sys.stderr);sys.exit(1)

 finally:
  if lock is not None:lock.rmdir()
