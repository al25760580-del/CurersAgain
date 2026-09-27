from pathlib import Path
import re,struct,json
ns={};exec(Path('/home/mila/gamemaker-analysis/scripts/annotate_visual.py').read_text().split("out=b/")[0],ns)
root=Path('/home/mila/gamemaker-analysis/ranking-cursor-native/exports-startup')
for f in root.glob('*.c'):
 if '.annotated.' in f.name:continue
 def sub(m):
  token=m[0];va=int(m[1],16);n=ns['name'](va)
  if n:return token+' /* '+n+' */'
  if 0x144f00000<=va<0x145100000:
   try:
    d=struct.unpack_from('<d',ns['pe'],ns['off'](va))[0]
    if 1e-8<abs(d)<1e8 or d==0:return token+' /* double '+repr(d)+' */'
   except:pass
  return token
 f.with_suffix('.annotated.c').write_text(re.sub(r'(?:_?DAT_|PTR_)([0-9a-f]{9})',sub,f.read_text()))
