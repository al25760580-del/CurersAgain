from pathlib import Path
import tomllib,json
p=Path('/home/mila/gamemaker-analysis/rust-port')
f=p/'src/lib.rs';s=f.read_text();s+='\npub mod translation_limits;\n';f.write_text(s)
f=p/'src/render/text.rs';s=f.read_text();at=s.index('    /// Draw one atlas-text line')
s=s[:at]+'''    fn glyph_extent(&self,ch:char)->(i32,i32,i32){
        if let Some((r,advance,offset))=self.glyphs.get(&ch){(*advance,*offset,r.width() as i32)}else{self.fallback.as_ref().map(|f|f.glyph_extent(ch)).unwrap_or((0,0,0))}
    }
    /// Includes advance and bitmap overhang, in logical pixels; same fallback as drawing.
    pub fn visual_width(&self,text:&str)->u32{
        let (mut pen,mut left,mut right)=(0i64,0i64,0i64);
        for ch in text.chars(){let (advance,offset,width)=self.glyph_extent(ch);if width>0 {left=left.min(pen+offset as i64);right=right.max(pen+offset as i64+width as i64);}pen+=advance as i64;right=right.max(pen);}
        (right-left).clamp(0,u32::MAX as i64) as u32
    }
'''+s[at:];f.write_text(s)
f=p/'src/bin/title_reference.rs';s=f.read_text();s=s.replace('    let mut scores_tiny = Font::load_language(&tc, &init_room_assets.language.tiny_font)?;','''    let mut scores_tiny = Font::load_language(&tc, &init_room_assets.language.tiny_font)?;
    holocure_core_port::translation_limits::warn(&language_root.join("layout-limits.toml"),&init_room_assets.language,&font,&scores_big,&scores_tiny);''');s=s.replace('                    scores_big = next_big;','''                    holocure_core_port::translation_limits::warn(&language_root.join("layout-limits.toml"),&pack,&next_font,&next_big,&next_tiny);
                    scores_big = next_big;''');f.write_text(s)
# Only remove the port's ad-hoc truncation of translated button labels, not player data.
f=p/'src/render/scores.rs';s=f.read_text().replace('''        clipped(
            c,
            font,
            label,
            118,''','''        text(
            c,
            font,
            label,''').replace('''                clipped(
                    c,
                    font,
                    &label,
                    170,''','''                text(
                    c,
                    font,
                    &label,''');f.write_text(s)
# Layout budgets are authoring recommendations, NOT recovered native string length checks.
lines=['# Authoring recommendations for the CURRENT port geometry. Not native input restrictions.', '# Warn only. max_chars is Unicode scalars per line; max_width is logical pixels.', '# A trailing * matches translated keys by prefix. Values are substitution examples.', 'schema = 1','']
def rule(key,chars,width,font='main',max_lines=1,values=None):
 lines.extend(['[[rules]]','key = '+json.dumps(key),f'max_chars = {chars}',f'max_width = {width}',f'max_lines = {max_lines}','font = '+json.dumps(font)])
 if values:
  lines.append('[rules.values]')
  for k,v in values.items():lines.append(k+' = '+json.dumps(v))
 lines.append('')
for key in ['check','my_score','quit','all_time','daily','character_*','stage_*']:rule('scores.'+key,19,118)
for key in ['show_names','use_name']:rule('scores.'+key,28,170,values={'value':['$scores.on','$scores.off']})
for key in ['back','settings','local_settings','close','details','details_limit']:rule('scores.'+key,35,210)
rule('scores.heading_*',29,354,'big')
for key in ['offline','cooldown','read_error','no_personal','empty']:rule('scores.'+key,59,354)
rule('scores.save_error',45,220,'tiny')
for key in ['show','hide','allow','decline','start','restart']:rule('first_run.'+key,24,140)
for key in ['language','name','show_prompt']:rule('first_run.'+key,90,560)
rule('first_run.warning',90,560,max_lines=2)
rule('first_run.consent',90,560,max_lines=5)
(p/'languages/layout-limits.toml').write_text('\n'.join(lines))
