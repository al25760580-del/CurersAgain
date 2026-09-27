//! Authoring diagnostics only. Limits never mutate, reject, or truncate translations.
use crate::{
    language::{parse, LanguagePack},
    render::text::Font,
};
use serde::Deserialize;
use std::{collections::BTreeMap, path::Path};
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Limits {
    pub schema: u32,
    pub rules: Vec<Rule>,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rule {
    pub key: String,
    pub max_chars: usize,
    pub max_width: u32,
    #[serde(default = "one")]
    pub max_lines: usize,
    #[serde(default)]
    pub font: Role,
    /// Substitution examples; "$key" refers to another translated string.
    #[serde(default)]
    pub values: BTreeMap<String, Vec<String>>,
}
fn one() -> usize {
    1
}
#[derive(Debug, Default, Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    #[default]
    Main,
    Big,
    Tiny,
}
#[derive(Debug, PartialEq)]
pub struct Warning {
    pub key: String,
    pub chars: usize,
    pub width: u32,
    pub lines: usize,
    pub max_chars: usize,
    pub max_width: u32,
    pub max_lines: usize,
}
impl Limits {
    pub fn load(path: &Path) -> Result<Self, String> {
        let limits: Self = parse(path)?;
        if limits.schema != 1
            || limits.rules.len() > 4096
            || limits.rules.iter().any(|r| {
                r.key.is_empty() || r.max_chars == 0 || r.max_width == 0 || r.max_lines == 0
            })
        {
            return Err("Invalid translation limits schema/rule".into());
        }
        Ok(limits)
    }
    pub fn check(
        &self,
        strings: &BTreeMap<String, String>,
        measure: impl Fn(Role, &str) -> u32,
    ) -> Vec<Warning> {
        let mut result = Vec::new();
        for rule in &self.rules {
            for (key, original) in strings {
                let matches = if let Some(prefix) = rule.key.strip_suffix('*') {
                    key.starts_with(prefix)
                } else {
                    key == &rule.key
                };
                if !matches {
                    continue;
                }
                let mut text = original.clone();
                let mut char_text = original.clone();
                for (name, values) in &rule.values {
                    let choices: Vec<&str> = values
                        .iter()
                        .map(|v| {
                            v.strip_prefix('$')
                                .and_then(|k| strings.get(k).map(String::as_str))
                                .unwrap_or(v.as_str())
                        })
                        .collect();
                    let wide = choices
                        .iter()
                        .max_by_key(|v| measure(rule.font, v))
                        .copied()
                        .unwrap_or("");
                    let long = choices
                        .iter()
                        .max_by_key(|v| v.chars().count())
                        .copied()
                        .unwrap_or("");
                    let placeholder = format!("{{{name}}}");
                    text = text.replace(&placeholder, wide);
                    char_text = char_text.replace(&placeholder, long);
                }
                let lines = text.split('\n').count();
                let chars = char_text
                    .split('\n')
                    .map(|l| l.chars().count())
                    .max()
                    .unwrap_or(0);
                let width = text
                    .split('\n')
                    .map(|l| measure(rule.font, l))
                    .max()
                    .unwrap_or(0);
                if chars > rule.max_chars || width > rule.max_width || lines > rule.max_lines {
                    result.push(Warning {
                        key: key.clone(),
                        chars,
                        width,
                        lines,
                        max_chars: rule.max_chars,
                        max_width: rule.max_width,
                        max_lines: rule.max_lines,
                    });
                }
            }
        }
        result
    }
}
/// Called only when loading/reloading a pack, never once per frame.
pub fn warn(path: &Path, pack: &LanguagePack, main: &Font, big: &Font, tiny: &Font) {
    match Limits::load(path) {
        Ok(limits) => {
            for w in limits.check(&pack.strings, |role, text| {
                match role {
                    Role::Main => main,
                    Role::Big => big,
                    Role::Tiny => tiny,
                }
                .visual_width(text)
            }) {
                eprintln!("TRANSLATION_LAYOUT_WARNING language={} key={} chars={}/{} width={}/{} logical_px lines={}/{} policy=warn_only",pack.manifest.id,w.key,w.chars,w.max_chars,w.width,w.max_width,w.lines,w.max_lines);
            }
        }
        Err(error) => eprintln!("TRANSLATION_LIMITS_WARNING {error}; translations unchanged"),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn rules() -> Limits {
        toml::from_str("schema=1\n[[rules]]\nkey='bubble.*'\nmax_chars=4\nmax_width=10\n").unwrap()
    }
    #[test]
    fn unicode_boundary_and_no_mutation() {
        let texts = BTreeMap::from([("bubble.a".into(), "áéñ好".into())]);
        let before = texts.clone();
        assert!(rules()
            .check(&texts, |_, s| s.chars().count() as u32 * 2)
            .is_empty());
        assert_eq!(texts, before);
    }
    #[test]
    fn wide_glyphs_and_multiline() {
        let texts = BTreeMap::from([("bubble.a".into(), "WW\na".into())]);
        let w = rules().check(&texts, |_, s| {
            s.chars().map(|c| if c == 'W' { 6 } else { 2 }).sum()
        });
        assert_eq!(w.len(), 1);
        assert_eq!((w[0].width, w[0].lines), (12, 2));
    }
    #[test]
    fn interpolated_translation_is_checked() {
        let mut r = rules();
        r.rules[0]
            .values
            .insert("value".into(), vec!["$on".into(), "$off".into()]);
        let texts = BTreeMap::from([
            ("bubble.a".into(), "{value}".into()),
            ("on".into(), "Enabled".into()),
            ("off".into(), "Disabled".into()),
        ]);
        assert_eq!(
            r.check(&texts, |_, s| s.chars().count() as u32)[0].chars,
            8
        );
    }
    #[test]
    fn unrelated_keys_are_not_limited() {
        assert!(rules()
            .check(
                &BTreeMap::from([("other".into(), "unlimited".into())]),
                |_, _| 100
            )
            .is_empty());
    }
}
