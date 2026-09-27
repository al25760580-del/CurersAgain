#!/usr/bin/env python3
"""Read-only view of the passive Steam inspector. Does not send game input.
Sprite rectangles describe drawing, NOT guaranteed hitboxes. Hover tests are
reported separately from original MouseOverButton return values.
"""
import argparse,json,time
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--snapshot',type=Path,default=Path('/home/mila/.local/share/Steam/steamapps/common/HoloCure/Logs/HoverInspector-latest.jsonl'));p.add_argument('--assets',type=Path,default=Path('/home/mila/gamemaker-analysis/asset-lab/sprites.json'));a=p.parse_args()
rows=[json.loads(l) for l in a.snapshot.read_text().splitlines() if l.strip()]
meta={s['name']:s['properties'] for s in json.loads(a.assets.read_text())}
f=next(r for r in rows if r['event']=='frame_begin');state=next(r for r in rows if r['event']=='ui_state')
mx,my=f['mouse_x'],f['mouse_y'];items=[];ui=False
for r in rows:
 if r.get('sprite')=='hud_optionsmenu':ui=True
 if not ui:continue
 if r['event']=='sprite':
  m=meta[r['sprite']];sx=r.get('xscale',r.get('width',m['Width'])/m['Width']);sy=r.get('yscale',r.get('height',m['Height'])/m['Height'])
  x=r['x']-m['OriginX']*sx;y=r['y']-m['OriginY']*sy;w=m['Width']*sx;h=m['Height']*sy
  items.append(dict(r,visual_rect=[x,y,w,h],mouse_inside_visual_rect=(x<=mx<x+w and y<=my<y+h),geometry_note='Unrotated sprite bounds; not an input hitbox.'))
 elif r['event'] in ('native_text','scribble_element'):items.append(r)
 elif r['event']=='script' and r.get('name','').startswith('draw_text'):
  items.append(r)
print(json.dumps({'snapshot_age_seconds':round(time.time()-a.snapshot.stat().st_mtime,3),'frame':f['frame'],'mouse':[mx,my],'state':state['vars'],'native_hover_tests':[r for r in rows if r['event']=='hover_test'],'settings_draw_items':items,'scope':'Observed obj_HiScores drawing only. No writes or input commands. Cached snapshot: check age and frame.'},ensure_ascii=False,indent=2))
