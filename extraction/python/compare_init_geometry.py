#!/usr/bin/env python3
"""Compare independently rendered Rust PNGs with preserved Steam captures.
Fixed, documented exclusions only: native OS cursor and English-language UI labels.
Pillow required. Does not write profiles or modify reference images.
Usage: python3 compare_init_geometry.py /path/to/lab/init-geometry
"""
import sys,json,hashlib
from pathlib import Path
from PIL import Image,ImageChops,ImageDraw
root=Path(sys.argv[1]);fixtures=json.loads((root/'comparison-fixtures.json').read_text());results=[]
for fixture in fixtures:
 step=fixture['initStep'];paths=[root/k/f'initStep-{step}.png' for k in ['native','after']]
 a,b=[Image.open(p).convert('RGB') for p in paths]
 assert a.size==b.size==(1280,720)
 channels=ImageChops.difference(a,b).split();diff=ImageChops.lighter(ImageChops.lighter(channels[0],channels[1]),channels[2]).point(lambda x:255 if x else 0)
 raw=diff.histogram()[255];mask=Image.new('L',a.size);pen=ImageDraw.Draw(mask)
 for exclusion in fixture['masks']:
  x,y,r,t=exclusion['box'];pen.rectangle((x,y,r-1,t-1),fill=255)
 diff.paste(0,mask=mask);remaining=diff.histogram()[255]
 results.append(dict(initStep=step,raw_different_pixels=raw,excluded_pixels=mask.histogram()[255],remaining_different_pixels=remaining,sha256=[hashlib.sha256(p.read_bytes()).hexdigest() for p in paths],masks=fixture['masks']))
(root/'comparison-results.json').write_text(json.dumps(results,indent=2)+'\n')
for result in results:print('initStep',result['initStep'],'raw differences',result['raw_different_pixels'],'remaining after explicit masks',result['remaining_different_pixels'])
sys.exit(1 if any(r['remaining_different_pixels'] for r in results) else 0)
