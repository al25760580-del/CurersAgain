//! Read-only reader for observed Steam 0.7 newSecureSave (base64-encoded JSON).
//! Not a legacy ds_map_secure_load implementation; no automatic migrations or writes.
use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::{Map, Value};
use std::{fs::File, io::Read, path::Path};
const MAX_BYTES: usize = 8 * 1024 * 1024;
#[derive(Debug)]
pub struct NativeSave {
    pub raw: Map<String, Value>,
}
#[derive(Debug)]
pub struct LocalScoreRef<'a> {
    pub stage: &'a str,
    pub character: &'a str,
    pub version_major: Option<&'a Value>,
    pub version_minor: Option<&'a Value>,
    pub all_time: Option<&'a Map<String, Value>>,
    pub daily_timestamp: Option<&'a Value>,
    pub daily_score: Option<&'a Map<String, Value>>,
}
fn read_bounded(path: &Path) -> Result<Vec<u8>, String> {
    let file = File::open(path).map_err(|e| format!("Cannot open native save: {e}"))?;
    let before = file.metadata().map_err(|e| e.to_string())?;
    if !before.is_file() || before.len() > MAX_BYTES as u64 {
        return Err("Native input must be a regular file of at most 8 MiB".into());
    }
    let mut bytes = Vec::new();
    (&file)
        .take((MAX_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    let after = file.metadata().map_err(|e| e.to_string())?;
    if bytes.len() > MAX_BYTES
        || before.len() != after.len()
        || before.modified().ok() != after.modified().ok()
        || bytes.len() as u64 != after.len()
    {
        return Err(
            "Native file changed during read or exceeded limit; retry with a stable copy".into(),
        );
    }
    Ok(bytes)
}
impl NativeSave {
    pub fn load(path: &Path) -> Result<Self, String> {
        Self::decode(&read_bounded(path)?)
    }
    pub fn decode(encoded: &[u8]) -> Result<Self, String> {
        if encoded.len() > MAX_BYTES {
            return Err("Native save exceeds 8 MiB".into());
        }
        // Tolerate outer ASCII whitespace, not arbitrary ignored bytes inside the stream.
        let start = encoded
            .iter()
            .position(|b| !b.is_ascii_whitespace())
            .unwrap_or(encoded.len());
        let end = encoded
            .iter()
            .rposition(|b| !b.is_ascii_whitespace())
            .map(|n| n + 1)
            .unwrap_or(start);
        let bytes = STANDARD.decode(&encoded[start..end]).map_err(|_| {
            "Not a supported base64 newSecureSave; legacy secure maps are not supported"
        })?;
        let value: Value = serde_json::from_slice(&bytes)
            .map_err(|_| "Decoded save is not valid supported JSON")?;
        let raw = value
            .as_object()
            .ok_or("Native save root must be an object")?
            .clone();
        Ok(Self { raw })
    }
    /// Borrow without normalizing floats, sentinel IDs, version numbers or unknown fields.
    pub fn field(&self, name: &str) -> Option<&Value> {
        self.raw.get(name)
    }
    pub fn local_scores(&self) -> Result<Vec<LocalScoreRef<'_>>, String> {
        let Some(scores) = self.raw.get("LocalScores") else {
            return Ok(vec![]);
        };
        let stages = scores.as_object().ok_or("LocalScores must be an object")?;
        let mut rows = Vec::new();
        for (stage, characters) in stages {
            for (character, entry) in characters
                .as_object()
                .ok_or("LocalScores stage must be an object")?
            {
                let e = entry
                    .as_object()
                    .ok_or("LocalScores character entry must be an object")?;
                let all_time = e
                    .get("allTime")
                    .map(|v| v.as_object().ok_or("allTime must be an object"))
                    .transpose()?;
                let daily = e
                    .get("daily")
                    .map(|v| v.as_object().ok_or("daily must be an object"))
                    .transpose()?;
                let daily_score = daily
                    .and_then(|d| d.get("score"))
                    .map(|v| v.as_object().ok_or("daily.score must be an object"))
                    .transpose()?;
                rows.push(LocalScoreRef {
                    stage,
                    character,
                    version_major: e.get("versionMajor"),
                    version_minor: e.get("versionMinor"),
                    all_time,
                    daily_timestamp: daily.and_then(|d| d.get("timestamp")),
                    daily_score,
                });
            }
        }
        Ok(rows)
    }
    /// Diagnostic metadata only: deliberately no usernames, score values, or auth data.
    pub fn summary(&self) -> Value {
        let fields:Map<String,Value>=self.raw.iter().map(|(k,v)|{
            let kind=match v{Value::Null=>"null",Value::Bool(_)=>"boolean",Value::Number(_)=>"number",Value::String(_)=>"string",Value::Array(_)=>"array",Value::Object(_)=>"object"};
            (k.clone(),serde_json::json!({"type":kind,"length":match v{Value::Array(a)=>Some(a.len()),Value::Object(o)=>Some(o.len()),_=>None}}))
        }).collect();
        serde_json::json!({"format":"base64-json-newSecureSave","field_count":self.raw.len(),"fields":fields,"local_score_groups":self.local_scores().ok().map(|a|a.len()),"read_only":true})
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn encoded(v: Value) -> String {
        STANDARD.encode(serde_json::to_vec(&v).unwrap())
    }
    #[test]
    fn preserves_numeric_flags_and_unknowns() {
        let v = serde_json::json!({"supports":1.0,"firstTime":true,"future":{"x":[null,false,2.5]},"GameVersionNumberMajor":0.7});
        let s = NativeSave::decode(encoded(v.clone()).as_bytes()).unwrap();
        assert_eq!(Value::Object(s.raw), v);
    }
    #[test]
    fn daily_nesting_and_versions() {
        let s=NativeSave::decode(encoded(serde_json::json!({"LocalScores":{"STAGE 1":{"ame":{"allTime":{"score":123.0,"weapons":"AA=="},"daily":{"timestamp":42.0,"score":{"score":5.0}},"versionMajor":0.7,"versionMinor":123.0}}}})).as_bytes()).unwrap();
        let rows = s.local_scores().unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].stage, "STAGE 1");
        assert_eq!(rows[0].all_time.unwrap()["weapons"], "AA==");
        assert_eq!(rows[0].daily_score.unwrap()["score"], 5.0);
    }
    #[test]
    fn absent_scores_are_not_fake_records() {
        assert!(
            NativeSave::decode(encoded(serde_json::json!({})).as_bytes())
                .unwrap()
                .local_scores()
                .unwrap()
                .is_empty()
        );
    }
    #[test]
    fn invalid_data_is_rejected() {
        for s in [
            b"not base64!".as_slice(),
            b"e30=garbage",
            b"W10=",
            b"bnVsbA==",
        ] {
            assert!(NativeSave::decode(s).is_err())
        }
        assert!(NativeSave::decode(&vec![b'A'; MAX_BYTES + 1]).is_err());
    }
    #[test]
    fn outer_whitespace_only() {
        assert!(NativeSave::decode(b"\r\ne30=\n").is_ok());
        assert!(NativeSave::decode(b"e 30=").is_err());
    }
    #[test]
    fn malformed_scores_not_silently_empty() {
        let s =
            NativeSave::decode(encoded(serde_json::json!({"LocalScores":[]})).as_bytes()).unwrap();
        assert!(s.local_scores().is_err());
    }
}
