//! Runtime TOML language packs. No compiled translations, scripts, network or font rasterizers.
use serde::Deserialize;
use std::{
    collections::{BTreeMap, HashSet},
    fs,
    path::{Component, Path, PathBuf},
};
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FontSpec {
    pub custom: bool,
    pub atlas: String,
    pub metrics: String,
    #[serde(default = "yes")]
    pub fallback_to_parent: bool,
}
fn yes() -> bool {
    true
}
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub schema: u32,
    pub id: String,
    pub name: String,
    pub native_name: String,
    pub text_format: String,
    pub texts: String,
    #[serde(default)]
    pub fallback: Option<String>,
    #[serde(default)]
    pub order: i32,
    #[serde(default)]
    pub font: Option<FontSpec>,
    #[serde(default)]
    pub big_font: Option<FontSpec>,
    #[serde(default)]
    pub tiny_font: Option<FontSpec>,
}
#[derive(Clone, Debug)]
pub struct FontSource {
    pub atlas: PathBuf,
    pub metrics: PathBuf,
    pub custom: bool,
    pub fallback: Option<Box<FontSource>>,
}
#[derive(Clone, Debug)]
pub struct LanguagePack {
    pub manifest: Manifest,
    pub strings: BTreeMap<String, String>,
    pub font: FontSource,
    pub big_font: FontSource,
    pub tiny_font: FontSource,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct TextFile {
    schema: u32,
    text_format: String,
    strings: BTreeMap<String, String>,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GlyphFile {
    pub schema: u32,
    pub format: String,
    pub name: String,
    pub atlas_width: u32,
    pub atlas_height: u32,
    pub init_baseline: f32,
    pub glyphs: BTreeMap<String, [i32; 6]>,
}
pub fn parse<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, String> {
    let meta = fs::metadata(path).map_err(|e| format!("{}: {e}", path.display()))?;
    if meta.len() > 4 * 1024 * 1024 {
        return Err(format!("Pack file exceeds 4 MiB: {}", path.display()));
    }
    toml::from_str(&fs::read_to_string(path).map_err(|e| e.to_string())?)
        .map_err(|e| format!("{}: {e}", path.display()))
}
fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 48
        && id
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_')
}
pub fn local_file(root: &Path, name: &str) -> Result<PathBuf, String> {
    let path = Path::new(name);
    if path.is_absolute()
        || path
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
    {
        return Err(format!(
            "Pack paths must be relative, without traversal: {name}"
        ));
    }
    let r = root.canonicalize().map_err(|e| e.to_string())?;
    let p = r.join(path).canonicalize().map_err(|e| e.to_string())?;
    if !p.starts_with(&r) {
        return Err("Pack symlink escapes its directory".into());
    }
    Ok(p)
}
fn manifest(root: &Path, id: &str) -> Result<(PathBuf, Manifest), String> {
    if !valid_id(id) {
        return Err("Invalid language ID".into());
    }
    let dir = local_file(root, id)?;
    let m: Manifest = parse(&local_file(&dir, "pack.toml")?)?;
    if m.schema != 1
        || m.text_format != "plain"
        || m.id != id
        || m.name.is_empty()
        || m.native_name.is_empty()
    {
        return Err(format!("Invalid or unsupported pack header: {id}"));
    }
    Ok((dir, m))
}
pub fn catalog(root: &Path) -> Result<Vec<Manifest>, String> {
    let mut packs = Vec::new();
    for item in fs::read_dir(root).map_err(|e| e.to_string())? {
        let p = item.map_err(|e| e.to_string())?.path();
        if p.is_dir() && p.join("pack.toml").is_file() {
            let id = p
                .file_name()
                .and_then(|n| n.to_str())
                .ok_or("Invalid pack directory")?;
            packs.push(manifest(root, id)?.1);
        }
    }
    packs.sort_by(|a, b| (a.order, &a.id).cmp(&(b.order, &b.id)));
    if packs.is_empty() || packs.len() > 5 {
        return Err("This native setup layout supports 1 to 5 installed language packs".into());
    }
    Ok(packs)
}
impl LanguagePack {
    pub fn load(root: &Path, id: &str) -> Result<Self, String> {
        Self::load_inner(root, id, &mut HashSet::new())
    }
    fn load_inner(root: &Path, id: &str, visited: &mut HashSet<String>) -> Result<Self, String> {
        if visited.len() >= 8 || !visited.insert(id.into()) {
            return Err("Language fallback cycle or excessive depth".into());
        }
        let (dir, m) = manifest(root, id)?;
        let parent = if let Some(id) = &m.fallback {
            Some(Self::load_inner(root, id, visited)?)
        } else {
            None
        };
        let t: TextFile = parse(&local_file(&dir, &m.texts)?)?;
        if t.schema != 1 || t.text_format != "plain" {
            return Err("Unsupported text schema/format; only plain UTF-8 is supported".into());
        }
        let mut strings = parent
            .as_ref()
            .map(|p| p.strings.clone())
            .unwrap_or_default();
        strings.extend(t.strings);
        let font = if let Some(spec) = &m.font {
            FontSource {
                atlas: local_file(&dir, &spec.atlas)?,
                metrics: local_file(&dir, &spec.metrics)?,
                custom: spec.custom,
                fallback: if spec.fallback_to_parent {
                    parent.as_ref().map(|p| Box::new(p.font.clone()))
                } else {
                    None
                },
            }
        } else {
            parent
                .as_ref()
                .ok_or("Base pack must define a font")?
                .font
                .clone()
        };
        let role = |spec: &Option<FontSpec>,
                    inherited: Option<&FontSource>|
         -> Result<FontSource, String> {
            if let Some(spec) = spec {
                Ok(FontSource {
                    atlas: local_file(&dir, &spec.atlas)?,
                    metrics: local_file(&dir, &spec.metrics)?,
                    custom: spec.custom,
                    fallback: if spec.fallback_to_parent {
                        inherited.map(|f| Box::new(f.clone()))
                    } else {
                        None
                    },
                })
            } else {
                Ok(inherited.unwrap_or(&font).clone())
            }
        };
        let big_font = role(&m.big_font, parent.as_ref().map(|p| &p.big_font))?;
        let tiny_font = role(&m.tiny_font, parent.as_ref().map(|p| &p.tiny_font))?;
        Ok(Self {
            big_font,
            tiny_font,
            manifest: m,
            strings,
            font,
        })
    }
    pub fn text<'a>(&'a self, key: &'a str) -> &'a str {
        self.strings.get(key).map(String::as_str).unwrap_or(key)
    }
    pub fn require(&self, keys: &[&str]) -> Result<(), String> {
        for key in keys {
            if !self.strings.contains_key(*key) {
                return Err(format!("Missing language key: {key}"));
            }
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_ids_and_paths() {
        assert!(!valid_id("../eng"));
        assert!(!valid_id("C:\\eng"));
        assert!(valid_id("pt-BR"));
        assert!(local_file(Path::new("."), "../test").is_err());
    }
    #[test]
    fn parses_literal_whitespace() {
        let t:TextFile=toml::from_str("schema=1\ntext_format='plain'\n[strings]\nconsent='''\nLine one\n\n You may change this setting later.'''\n").unwrap();
        assert!(t.strings["consent"].contains("\n\n You"));
    }
    #[test]
    fn rejects_unknown_header_fields() {
        assert!(toml::from_str::<TextFile>(
            "schema=1\ntext_format='plain'\nexecute='bad'\n[strings]\n"
        )
        .is_err());
    }
    #[test]
    fn external_base_is_complete() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("languages");
        let p = LanguagePack::load(&root, "eng").unwrap();
        p.require(&["first_run.consent", "title.play", "startup.intro.0"])
            .unwrap();
        assert!(p.text("first_run.consent").contains("\n\n You"));
        let inherited = LanguagePack::load(&root, "jp").unwrap();
        assert_eq!(p.text("title.play"), inherited.text("title.play"));
    }
}
