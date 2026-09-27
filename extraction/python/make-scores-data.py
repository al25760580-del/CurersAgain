from pathlib import Path
import json,re,shutil
r=Path('/home/mila/gamemaker-analysis');p=r/'rust-port';a=r/'asset-lab'
manifest=json.loads((a/'title-manifest.json').read_text());old=(r/'legacy-android/export/gml/gml_Object_obj_TextController_Create_0.gml').read_text()
names=dict(re.findall(r'(\w+)Name:\s*\{\s*eng:\s*"([^"\n]+)"',old))
chars={v['sprite'].removeprefix('spr_Title_').lower():v['sprite'] for v in manifest['characters']}
order=['ame','gura','ina','kiara','calli','bae','kronii','fauna','mumei','sana','irys','fubuki','mio','okayu','korone','sora','azki','roboco','suisei','miko','haato','mel','matsuri','aki','subaru','choco','shion','ayame','aqua']
order=[c for c in order if c in chars]+[c for c in chars if c not in order]
config='schema = 1\n'
en={
'scores.all_time':'All Time','scores.daily':'Daily','scores.empty':'No scores for these filters.',
'scores.check':'Check Scores','scores.my_score':'My Score','scores.quit':'Quit','scores.rank':'Rank','scores.score_value':'Score: {score}','scores.run_values':'TIME: {time}  LV: {level}','scores.anonymous':'Anonymous','scores.controls':'PAGE: LEFT/RIGHT | CONFIRM: SPACE/ENTER | BACK: SHIFT/ESC',
'scores.settings':'SETTINGS','scores.show_names':'Show Names: {value}','scores.use_name':'Use My Name: {value}','scores.on':'On','scores.off':'Off','scores.back':'Back','scores.local_settings':'Local preferences only','scores.details':'SCORE DETAILS','scores.details_limit':'Equipment data is not imported.','scores.close':'ENTER / ESC: CLOSE','scores.all_characters':'All Characters'}
es={'scores.all_time':'Todos los tiempos','scores.daily':'Diario','scores.empty':'No hay puntuaciones con estos filtros.','scores.check':'Consultar','scores.my_score':'Mi puntuación','scores.quit':'Volver','scores.rank':'Puesto','scores.score_value':'Puntos: {score}','scores.run_values':'TIEMPO: {time}  NV: {level}','scores.anonymous':'Anónimo','scores.controls':'PÁGINA: IZQ/DER | ACEPTAR: ESPACIO/ENTER | VOLVER: SHIFT/ESC','scores.settings':'AJUSTES','scores.show_names':'Ver nombres: {value}','scores.use_name':'Usar mi nombre: {value}','scores.on':'Sí','scores.off':'No','scores.back':'Volver','scores.local_settings':'Solo preferencias locales','scores.details':'DETALLES','scores.details_limit':'No se importó el equipamiento.','scores.close':'ENTER / ESC: CERRAR','scores.all_characters':'Todos los personajes','title.leaderboards':'Puntuaciones'}
# Resource-backed stage choices; exact native ordering/unlock gating is not claimed.
for i in range(1,6):
 id=f'STAGE {i}';key=f'scores.stage_{i}';config+=f'\n[[stages]]\nid={json.dumps(id)}\ntext_key="{key}"\n';en[key]=id;es[key]=f'ESCENARIO {i}'
for i in range(1,5):
 id=f'STAGE {i} (HARD)';key=f'scores.stage_{i}_hard';config+=f'\n[[stages]]\nid={json.dumps(id)}\ntext_key="{key}"\n';en[key]=id;es[key]=f'ESC. {i} (DIFÍCIL)'
for id in order:
 key='scores.character_'+id;config+=f'\n[[characters]]\nid="{id}"\ntext_key="{key}"\nsprite="{chars[id]}"\n';en[key]=names.get(id,id.upper())
config+='\n[[characters]]\nid="all"\ntext_key="scores.all_characters"\n'
(a/'scores-config.toml').write_text(config)
for lang,values in [('eng',en),('es',es)]:
 file=p/'languages'/lang/'texts.toml'
 if not file.exists():continue
 text=file.read_text()
 for key,value in values.items():
  # Replace existing translated title key; append genuinely new score keys.
  pattern=r'^'+re.escape(json.dumps(key))+r' = .*?$'
  line=json.dumps(key)+' = '+json.dumps(value,ensure_ascii=False)
  if re.search(pattern,text,re.M):text=re.sub(pattern,lambda m:line,text,flags=re.M)
  else:text+='\n'+line
 file.write_text(text+'\n')
 if lang=='es' and (p/'language-examples/es').exists():shutil.copy2(file,p/'language-examples/es/texts.toml')
example={'schema':1,'records':[{'username':f'Demo {i+1:02}','stage':'STAGE 1','character':'ame','score':20000000-i*150000,'duration_seconds':1800+i*17,'level':200-i,'daily':i%2==0,'is_player':i==12} for i in range(23)]}
(p/'examples').mkdir(exist_ok=True);(p/'examples/scores.demo.json').write_text(json.dumps(example,indent=2))
print('SCORES_CATALOG',len(order),'characters + all; demonstration fixture is NOT installed in player profile')
