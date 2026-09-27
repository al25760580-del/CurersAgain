from pathlib import Path
import shutil,datetime,json
root=Path('/home/mila/gamemaker-analysis');p=root/'rust-port';b=p/'backups'/('before-init-geometry-'+datetime.datetime.now(datetime.timezone.utc).strftime('%Y%m%dT%H%M%SZ'));b.mkdir(parents=True)
for f in ['src/first_run.rs','src/render/first_run.rs','src/render/text.rs','src/bin/title_reference.rs','tests/first_run_sdl.rs']:
 t=b/f;t.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(p/f,t)
lab=root/'lab/init-geometry';lab.mkdir(parents=True,exist_ok=True)
shutil.copytree(root/'lab/first-run-render',lab/'before',dirs_exist_ok=True)
native=lab/'native';native.mkdir(exist_ok=True)
original=root/'backups/fresh-save-20260927T071921Z/observation/attempt2-preserved'
for step,num in enumerate([3,8,9,10,12]):shutil.copy2(original/f'{num:03}.png',native/f'initStep-{step}.png')
(lab/'manifest.json').write_text(json.dumps({'backup':str(b),'source_numbers':[3,8,9,10,12],'native_source':str(original),'note':'Preserved Steam captures, not a new native run. Port captures are independent SDL renders.'},indent=2))
s=(p/'src/first_run.rs').read_text();a=s.index('        let y = match self.initStep {',s.index('pub fn button_rect'));z=s.index('\n    pub fn MouseOverButton',a)
s=s[:a]+'''        // Steam Draw_64: origin (320,y), long hitbox +-90, 29 high, stride 34.
        // Selected/unselected sprite alpha differs; the hitbox does not shrink.
        let y=match self.initStep {0|2=>156,3=>171,_=>226};
        (230,y+i as i32*34,180,29)
    }
'''+s[z:];s=s.replace('vec!["Start", "Restart"]','vec!["Start!", "Restart"]');(p/'src/first_run.rs').write_text(s)
s=(p/'src/render/text.rs').read_text();pos=s.index('    /// Draw one atlas-text line');s=s[:pos]+'''    /// GameMaker string_width for the supported atlas glyphs (logical pixels).
    pub fn width(&self,text:&str)->i32 {text.chars().filter_map(|ch|self.glyphs.get(&ch).map(|g|g.1)).sum()}
'''+s[pos:];(p/'src/render/text.rs').write_text(s)
s=(p/'src/bin/title_reference.rs').read_text();pos=s.index('    let mut high_resolution =');s=s[:pos]+'''    let init_room_assets=holocure_core_port::render::first_run::InitRoomAssets::load(&tc,&root)?;
    println!("FIRST_RUN_OFFLINE: local settings only; no scores or personal data are sent. UI remains English regardless of saved language preference.");
'''+s[pos:];s=s.replace('first_run::draw(c, &mut font, &GameManager)','first_run::draw(c, &mut font, &GameManager, &init_room_assets)');(p/'src/bin/title_reference.rs').write_text(s)
s=(p/'tests/first_run_sdl.rs').read_text();s=s.replace('let mut g = GameManager::new(Settings::default());','let assets=first_run::InitRoomAssets::load(&tc,&root).unwrap();\n    let mut g = GameManager::new(Settings::default());');s=s.replace('g.settings.username = "Test Player".into();','g.settings.username = "MilaTestPlayerName".into();\n    g.inputText=g.settings.username.clone();g.settings.hiscoreName=true;g.rectVis=true;');s=s.replace('first_run::draw(&mut c, &mut font, &g)','first_run::draw(&mut c, &mut font, &g, &assets)');(p/'tests/first_run_sdl.rs').write_text(s)
shutil.copy2(root/'scripts/init-geometry-render.rs',p/'src/render/first_run.rs')
print('BACKUP',b)
