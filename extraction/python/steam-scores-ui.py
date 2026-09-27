"""Bounded manual research actions on the exact Steam HoloCure X11 window.
No calls into GameMaker, no memory writes, no automatic traversal.
"""
import ctypes as c,sys,time
from pathlib import Path
from PIL import ImageGrab
L=c.CDLL('libX11.so.6');T=c.CDLL('libXtst.so.6');P=c.c_void_p;U=c.c_ulong
L.XOpenDisplay.restype=P;L.XOpenDisplay.argtypes=[c.c_char_p];d=L.XOpenDisplay(None)
if not d:raise SystemExit('No display')
L.XDefaultRootWindow.argtypes=[P];L.XDefaultRootWindow.restype=U;root=L.XDefaultRootWindow(d)
L.XFetchName.argtypes=[P,U,c.POINTER(c.c_char_p)];L.XFree.argtypes=[P]
L.XQueryTree.argtypes=[P,U,c.POINTER(U),c.POINTER(U),c.POINTER(c.POINTER(U)),c.POINTER(c.c_uint)]
def find(w,depth=0):
 name=c.c_char_p()
 if L.XFetchName(d,w,c.byref(name)) and name.value:
  match=name.value==b'HoloCure';L.XFree(name)
  if match:return w
 if depth>5:return None
 r=U();p=U();kids=c.POINTER(U)();n=c.c_uint()
 if L.XQueryTree(d,w,c.byref(r),c.byref(p),c.byref(kids),c.byref(n)):
  ids=list(kids[:n.value]);L.XFree(kids)
  for k in ids:
   v=find(k,depth+1)
   if v:return v
w=find(root)
if not w:raise SystemExit('Steam HoloCure window not found')
L.XTranslateCoordinates.argtypes=[P,U,U,c.c_int,c.c_int,c.POINTER(c.c_int),c.POINTER(c.c_int),c.POINTER(U)]
x=c.c_int();y=c.c_int();child=U();L.XTranslateCoordinates(d,w,root,0,0,c.byref(x),c.byref(y),c.byref(child))
L.XGetGeometry.argtypes=[P,U,c.POINTER(U),c.POINTER(c.c_int),c.POINTER(c.c_int),c.POINTER(c.c_uint),c.POINTER(c.c_uint),c.POINTER(c.c_uint),c.POINTER(c.c_uint)]
groot=U();gx=c.c_int();gy=c.c_int();width=c.c_uint();height=c.c_uint();border=c.c_uint();depth=c.c_uint();L.XGetGeometry(d,w,c.byref(groot),c.byref(gx),c.byref(gy),c.byref(width),c.byref(height),c.byref(border),c.byref(depth))
L.XFlush.argtypes=[P];L.XSetInputFocus.argtypes=[P,U,c.c_int,U]
T.XTestFakeKeyEvent.argtypes=[P,c.c_uint,c.c_int,U];T.XTestFakeButtonEvent.argtypes=[P,c.c_uint,c.c_int,U];T.XTestFakeMotionEvent.argtypes=[P,c.c_int,c.c_int,c.c_int,U]
L.XStringToKeysym.argtypes=[c.c_char_p];L.XStringToKeysym.restype=U;L.XKeysymToKeycode.argtypes=[P,U];L.XKeysymToKeycode.restype=c.c_uint
# Activate the top-level game through the WM, rather than raising only a child.
class Data(c.Union):_fields_=[('l',c.c_long*5)]
class Client(c.Structure):_fields_=[('type',c.c_int),('serial',U),('send_event',c.c_int),('display',P),('window',U),('message_type',U),('format',c.c_int),('data',Data)]
class Event(c.Union):_fields_=[('client',Client),('pad',c.c_long*24)]
L.XInternAtom.argtypes=[P,c.c_char_p,c.c_int];L.XInternAtom.restype=U
L.XSendEvent.argtypes=[P,U,c.c_int,c.c_long,c.POINTER(Event)]
e=Event();e.client.type=33;e.client.display=d;e.client.window=w;e.client.message_type=L.XInternAtom(d,b'_NET_ACTIVE_WINDOW',0);e.client.format=32;e.client.data.l[0]=2
L.XSendEvent(d,root,0,(1<<20)|(1<<19),c.byref(e));L.XFlush(d);time.sleep(.12)
cmd=sys.argv[1]
if cmd in ('key','click','move'):
 L.XSetInputFocus(d,w,2,0);L.XFlush(d)
 if cmd=='key':
  key=sys.argv[2]
  allowed=['Left','Right','Up','Down','Return','Escape','Tab','Shift_L','Shift_R']
  if any(k not in allowed for k in key.split('+')):raise SystemExit('Key not allowed')
  codes=[L.XKeysymToKeycode(d,L.XStringToKeysym(k.encode())) for k in key.split('+')]
  for code in codes:T.XTestFakeKeyEvent(d,code,1,0);L.XFlush(d);time.sleep(.12)
  for code in reversed(codes):T.XTestFakeKeyEvent(d,code,0,0);L.XFlush(d);time.sleep(.1)
 else:
  px,py=map(float,sys.argv[2:4])
  if not (0<=px<640 and 0<=py<360):raise SystemExit('Coordinates outside logical game view')
  T.XTestFakeMotionEvent(d,-1,x.value+int(px*width.value/640),y.value+int(py*height.value/360),0);L.XFlush(d)
  if cmd=='click':
   time.sleep(.08);T.XTestFakeButtonEvent(d,1,1,0);L.XFlush(d);time.sleep(.055);T.XTestFakeButtonEvent(d,1,0,0)
 L.XFlush(d);time.sleep(.25)
elif cmd!='shot':raise SystemExit('Unknown action')
p=Path('/home/mila/gamemaker-analysis/lab/scores-native/steam-current.png');ImageGrab.grab(bbox=(x.value,y.value,x.value+width.value,y.value+height.value)).save(p)
print('STEAM_UI',cmd,'window',hex(w),'size',width.value,height.value,'screenshot',p)
