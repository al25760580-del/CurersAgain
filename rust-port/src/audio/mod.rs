//! Local PCM playback for verified ordinary startup/title cues, not the full GM audio engine.
//! Native PreIntro.Create 0x142d9c6b0 stops all audio. Intro.Create 0x142528f50
//! plays sound 18 (bgm_intro), priority 0, no loop. Title.Create 0x1435bd370
//! replaces it with sound 20 (bgm_SSS), looped. Menu select=239, confirm=238.
//! Original assets are decoded offline to interleaved stereo f32 LE at 48 kHz.
//! Gain here is a port setting; original saved volume, audio groups, DSP and exact device latency
//! are NOT reconstructed. The callback advances on the audio clock, not the drawing FPS.
use crate::scenes::startup::BootRoom;
use sdl2::audio::{AudioCallback, AudioDevice, AudioSpecDesired};
use std::{path::Path, sync::Arc};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cue {
    Intro,
    Title,
    Select,
    Confirm,
    Back,
}
#[derive(Clone)]
struct Clip(Arc<Vec<f32>>);
struct Voice {
    clip: Clip,
    cursor: usize,
    looped: bool,
}
impl Voice {
    fn sample(&mut self) -> f32 {
        if self.cursor >= self.clip.0.len() {
            if self.looped {
                self.cursor = 0;
            } else {
                return 0.0;
            }
        }
        let v = self.clip.0[self.cursor];
        self.cursor += 1;
        v
    }
    fn done(&self) -> bool {
        !self.looped && self.cursor >= self.clip.0.len()
    }
}
/// Pure mixer state. No disk I/O, decoding or allocations inside the SDL callback.
pub struct Mixer {
    music: Option<Voice>,
    voices: Vec<Voice>,
    gain: f32,
    muted: bool,
}
impl Mixer {
    fn new(gain: f32) -> Self {
        Self {
            music: None,
            voices: Vec::with_capacity(16),
            gain,
            muted: false,
        }
    }
    fn mix(&mut self, out: &mut [f32]) {
        for dst in out {
            let mut x = self.music.as_mut().map_or(0.0, |v| v.sample());
            for v in &mut self.voices {
                x += v.sample();
            }
            *dst = if self.muted {
                0.0
            } else {
                (x * self.gain).clamp(-1.0, 1.0)
            };
        }
        self.voices.retain(|v| !v.done());
        if self.music.as_ref().is_some_and(|v| v.done()) {
            self.music = None;
        }
    }
}
impl AudioCallback for Mixer {
    type Channel = f32;
    fn callback(&mut self, out: &mut [f32]) {
        self.mix(out);
    }
}
/// SDL adapter owning decoded clips and the device. Scene changes are idempotent.
pub struct TitleAudio {
    device: AudioDevice<Mixer>,
    clips: [Clip; 5],
    scene: Option<BootRoom>,
}
impl TitleAudio {
    /// Open a 48-kHz stereo output. A caller can explicitly fall back to silent mode on error.
    pub fn open(sdl: &sdl2::Sdl, root: &Path, gain: f32) -> Result<Self, String> {
        let load = |name: &str| -> Result<Clip, String> {
            let p = root.join("audio").join(format!("{name}.f32"));
            let bytes = std::fs::read(&p).map_err(|e| format!("{}: {e}", p.display()))?;
            if bytes.is_empty() || bytes.len() % 8 != 0 {
                return Err(format!("Invalid stereo PCM: {}", p.display()));
            }
            let pcm = bytes
                .chunks_exact(4)
                .map(|b| f32::from_le_bytes(b.try_into().unwrap()))
                .collect::<Vec<_>>();
            if !pcm.iter().all(|v| v.is_finite()) {
                return Err("Non-finite PCM".into());
            }
            Ok(Clip(Arc::new(pcm)))
        };
        let clips = [
            load("bgm_intro")?,
            load("bgm_SSS")?,
            load("snd_menu_select")?,
            load("snd_menu_confirm")?,
            load("snd_menu_back")?,
        ];
        let subsystem = sdl.audio()?;
        let desired = AudioSpecDesired {
            freq: Some(48000),
            channels: Some(2),
            samples: Some(1024),
        };
        let device =
            subsystem.open_playback(None, &desired, |_| Mixer::new(gain.clamp(0.0, 1.0)))?;
        if device.spec().freq != 48000 || device.spec().channels != 2 {
            return Err("Audio format differs from decoded assets".into());
        }
        device.resume();
        Ok(Self {
            device,
            clips,
            scene: None,
        })
    }
    /// Replace only on room change; no BGM restart at every frame. PreIntro also clears SFX.
    pub fn room(&mut self, room: BootRoom) {
        if self.scene == Some(room) {
            return;
        }
        self.scene = Some(room);
        let mut m = self.device.lock();
        m.voices.clear();
        m.music = match room {
            BootRoom::PreIntro => None,
            BootRoom::Intro => Some(Voice {
                clip: self.clips[0].clone(),
                cursor: 0,
                looped: false,
            }),
            BootRoom::Title => Some(Voice {
                clip: self.clips[1].clone(),
                cursor: 0,
                looped: true,
            }),
        };
    }
    /// Play one verified menu cue, with bounded polyphony; called on an action, not each Draw.
    pub fn cue(&mut self, cue: Cue) {
        let index = match cue {
            Cue::Select => 2,
            Cue::Confirm => 3,
            Cue::Back => 4,
            _ => return,
        };
        let mut m = self.device.lock();
        if m.voices.len() == 16 {
            m.voices.remove(0);
        }
        m.voices.push(Voice {
            clip: self.clips[index].clone(),
            cursor: 0,
            looped: false,
        });
    }
    /// Mute preserves timeline; debug pause freezes the device separately.
    pub fn toggle_mute(&mut self) -> bool {
        let mut m = self.device.lock();
        m.muted = !m.muted;
        m.muted
    }
    pub fn pause(&self, paused: bool) {
        if paused {
            self.device.pause();
        } else {
            self.device.resume();
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn clip(v: &[f32]) -> Clip {
        Clip(Arc::new(v.to_vec()))
    }
    #[test]
    fn one_shot_finishes_and_music_loops() {
        let mut m = Mixer::new(1.0);
        m.music = Some(Voice {
            clip: clip(&[0.1, 0.2]),
            cursor: 0,
            looped: true,
        });
        m.voices.push(Voice {
            clip: clip(&[0.3, 0.4]),
            cursor: 0,
            looped: false,
        });
        let mut out = [0.0; 4];
        m.mix(&mut out);
        assert_eq!(out, [0.4, 0.6, 0.1, 0.2]);
        assert!(m.voices.is_empty());
    }
    #[test]
    fn muted_audio_still_advances() {
        let mut m = Mixer::new(1.0);
        m.music = Some(Voice {
            clip: clip(&[0.1, 0.2, 0.3, 0.4]),
            cursor: 0,
            looped: false,
        });
        m.muted = true;
        let mut out = [1.0; 2];
        m.mix(&mut out);
        assert_eq!(out, [0.0, 0.0]);
        m.muted = false;
        m.mix(&mut out);
        assert_eq!(out, [0.3, 0.4]);
        assert!(m.music.is_none());
    }
    #[test]
    fn mixed_signal_clips_safely() {
        let mut m = Mixer::new(1.0);
        m.voices.push(Voice {
            clip: clip(&[2.0, -2.0]),
            cursor: 0,
            looped: false,
        });
        let mut out = [0.0; 2];
        m.mix(&mut out);
        assert_eq!(out, [1.0, -1.0]);
    }
}
