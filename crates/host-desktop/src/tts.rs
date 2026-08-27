use std::sync::{mpsc, Mutex};
use std::thread;
use std::time::Duration;

#[cfg(not(target_os = "macos"))]
use std::io;
#[cfg(not(target_os = "macos"))]
use std::process::{Child, Command, Stdio};
#[cfg(not(target_os = "macos"))]
use std::sync::atomic::{AtomicBool, Ordering};

use serde::{Deserialize, Serialize};

use vrcx_0_platform::Error;

#[cfg(windows)]
use base64::{engine::general_purpose::STANDARD as B64, Engine};

pub const DEFAULT_TTS_VOLUME: u8 = 100;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct TtsVoice {
    pub id: String,
    pub name: String,
    pub language: String,
}

pub trait TtsEngine: Send + Sync {
    fn voices(&self) -> Vec<TtsVoice>;

    fn speak(&self, text: &str, voice_id: Option<&str>, volume: u8) -> Result<(), Error>;
}

#[derive(Debug)]
struct TtsRequest {
    text: String,
    voice_id: Option<String>,
    volume: u8,
}

pub struct SystemTtsEngine {
    sender: Mutex<mpsc::Sender<TtsRequest>>,
}

impl Default for SystemTtsEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemTtsEngine {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        if let Err(error) = thread::Builder::new()
            .name("vrcx-0-tts".into())
            .spawn(move || run_tts_worker(receiver))
        {
            tracing::warn!("failed to start TTS worker: {error}");
        }
        Self {
            sender: Mutex::new(sender),
        }
    }
}

impl TtsEngine for SystemTtsEngine {
    fn voices(&self) -> Vec<TtsVoice> {
        platform_voices()
    }

    fn speak(&self, text: &str, voice_id: Option<&str>, volume: u8) -> Result<(), Error> {
        let request = TtsRequest {
            text: text.to_string(),
            voice_id: voice_id
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToString::to_string),
            volume: volume.min(DEFAULT_TTS_VOLUME),
        };
        let sender = self
            .sender
            .lock()
            .map_err(|error| Error::Custom(format!("TTS worker lock poisoned: {error}")))?;
        sender
            .send(request)
            .map_err(|error| Error::Custom(format!("TTS worker unavailable: {error}")))
    }
}

#[cfg(not(target_os = "macos"))]
fn run_tts_worker(receiver: mpsc::Receiver<TtsRequest>) {
    let mut child = None;
    loop {
        let request = if child.is_some() {
            match receiver.try_recv() {
                Ok(request) => Some(request),
                Err(mpsc::TryRecvError::Empty) => None,
                Err(mpsc::TryRecvError::Disconnected) => break,
            }
        } else {
            match receiver.recv() {
                Ok(request) => Some(request),
                Err(_) => break,
            }
        };

        if let Some(request) = request {
            stop_child(&mut child);
            if !request.text.trim().is_empty() && request.volume > 0 {
                match spawn_tts_child(&request.text, request.voice_id.as_deref(), request.volume) {
                    Ok(next) => child = Some(next),
                    Err(error) => warn_tts_spawn_once(&error),
                }
            }
        }

        if let Some(current) = child.as_mut() {
            match current.try_wait() {
                Ok(Some(_)) => child = None,
                Ok(None) => thread::sleep(Duration::from_millis(50)),
                Err(error) => {
                    warn_tts_spawn_once(&error);
                    child = None;
                }
            }
        }
    }
    stop_child(&mut child);
}

#[cfg(target_os = "macos")]
fn run_tts_worker(receiver: mpsc::Receiver<TtsRequest>) {
    use objc2::rc::autoreleasepool;
    use objc2_avf_audio::{
        AVSpeechBoundary, AVSpeechSynthesisVoice, AVSpeechSynthesizer, AVSpeechUtterance,
    };
    use objc2_foundation::NSString;

    let synthesizer = autoreleasepool(|_| unsafe { AVSpeechSynthesizer::new() });
    loop {
        let request = if unsafe { synthesizer.isSpeaking() } {
            match receiver.try_recv() {
                Ok(request) => Some(request),
                Err(mpsc::TryRecvError::Empty) => None,
                Err(mpsc::TryRecvError::Disconnected) => break,
            }
        } else {
            match receiver.recv() {
                Ok(request) => Some(request),
                Err(_) => break,
            }
        };

        if let Some(request) = request {
            unsafe {
                synthesizer.stopSpeakingAtBoundary(AVSpeechBoundary::Immediate);
            }
            if !request.text.trim().is_empty() && request.volume > 0 {
                autoreleasepool(|_| unsafe {
                    let text = NSString::from_str(&request.text);
                    let utterance = AVSpeechUtterance::speechUtteranceWithString(&text);
                    utterance.setVolume(f32::from(request.volume) / 100.0);
                    if let Some(voice_id) = request.voice_id.as_deref() {
                        let voices = AVSpeechSynthesisVoice::speechVoices();
                        if let Some(voice) = voices
                            .iter()
                            .find(|voice| voice.name().to_string() == voice_id)
                        {
                            utterance.setVoice(Some(&voice));
                        }
                    }
                    synthesizer.speakUtterance(&utterance);
                });
            }
        }

        if unsafe { synthesizer.isSpeaking() } {
            thread::sleep(Duration::from_millis(50));
        }
    }
    unsafe {
        synthesizer.stopSpeakingAtBoundary(AVSpeechBoundary::Immediate);
    }
}

#[cfg(not(target_os = "macos"))]
fn stop_child(child: &mut Option<Child>) {
    if let Some(mut current) = child.take() {
        let _ = current.kill();
        let _ = current.wait();
    }
}

#[cfg(not(target_os = "macos"))]
fn warn_tts_spawn_once(error: &io::Error) {
    static WARNED: AtomicBool = AtomicBool::new(false);
    if !WARNED.swap(true, Ordering::SeqCst) {
        tracing::warn!("native TTS command failed: {error}");
    }
}

#[cfg(windows)]
fn spawn_tts_child(text: &str, voice_id: Option<&str>, volume: u8) -> io::Result<Child> {
    let text_b64 = B64.encode(text.as_bytes());
    let voice_b64 = B64.encode(voice_id.unwrap_or_default().as_bytes());
    let script = format!(
        r#"
Add-Type -AssemblyName System.Speech
$text = [Text.Encoding]::UTF8.GetString([Convert]::FromBase64String('{text_b64}'))
$voice = [Text.Encoding]::UTF8.GetString([Convert]::FromBase64String('{voice_b64}'))
$speaker = New-Object System.Speech.Synthesis.SpeechSynthesizer
try {{
    if ($voice.Trim().Length -gt 0) {{
        try {{ $speaker.SelectVoice($voice) }} catch {{ }}
    }}
    $speaker.Volume = {volume}
    $speaker.Speak($text) | Out-Null
}} finally {{
    $speaker.Dispose()
}}
"#
    );
    spawn_powershell_script(&script)
}

#[cfg(all(not(windows), not(target_os = "macos")))]
fn spawn_tts_child(text: &str, _voice_id: Option<&str>, volume: u8) -> io::Result<Child> {
    Command::new("spd-say")
        .args(["--volume", &speech_dispatcher_volume(volume).to_string()])
        .arg("--")
        .arg(text)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
}

#[cfg(any(all(not(windows), not(target_os = "macos")), test))]
fn speech_dispatcher_volume(volume: u8) -> i16 {
    i16::from(volume.min(DEFAULT_TTS_VOLUME)) * 2 - 100
}

#[cfg(windows)]
fn platform_voices() -> Vec<TtsVoice> {
    let script = r#"
Add-Type -AssemblyName System.Speech
$speaker = New-Object System.Speech.Synthesis.SpeechSynthesizer
try {
    @($speaker.GetInstalledVoices() | ForEach-Object {
        $info = $_.VoiceInfo
        [pscustomobject]@{
            id = $info.Name
            name = $info.Name
            language = $info.Culture.Name
        }
    }) | ConvertTo-Json -Compress
} finally {
    $speaker.Dispose()
}
"#;
    match powershell_output(script) {
        Ok(output) => parse_windows_voices_json(&output).unwrap_or_default(),
        Err(error) => {
            tracing::debug!("failed to list native TTS voices: {error}");
            Vec::new()
        }
    }
}

#[cfg(target_os = "macos")]
fn platform_voices() -> Vec<TtsVoice> {
    use objc2::rc::autoreleasepool;
    use objc2_avf_audio::AVSpeechSynthesisVoice;

    autoreleasepool(|_| unsafe {
        AVSpeechSynthesisVoice::speechVoices()
            .iter()
            .map(|voice| TtsVoice {
                id: voice.name().to_string(),
                name: voice.name().to_string(),
                language: voice.language().to_string(),
            })
            .collect()
    })
}

#[cfg(all(not(windows), not(target_os = "macos")))]
fn platform_voices() -> Vec<TtsVoice> {
    Vec::new()
}

#[cfg(windows)]
fn powershell_output(script: &str) -> io::Result<Vec<u8>> {
    let output = powershell_command(script).output()?;
    if output.status.success() {
        Ok(output.stdout)
    } else {
        Err(io::Error::other(
            String::from_utf8_lossy(&output.stderr).trim().to_string(),
        ))
    }
}

#[cfg(windows)]
fn spawn_powershell_script(script: &str) -> io::Result<Child> {
    powershell_command(script)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
}

#[cfg(windows)]
fn powershell_command(script: &str) -> Command {
    use std::os::windows::process::CommandExt;

    const CREATE_NO_WINDOW: u32 = 0x08000000;

    let mut bytes = Vec::with_capacity(script.len() * 2);
    for unit in script.encode_utf16() {
        bytes.extend_from_slice(&unit.to_le_bytes());
    }
    let mut command = Command::new("powershell.exe");
    command.args([
        "-NoProfile",
        "-NonInteractive",
        "-ExecutionPolicy",
        "Bypass",
        "-EncodedCommand",
        &B64.encode(bytes),
    ]);
    command.creation_flags(CREATE_NO_WINDOW);
    command
}

#[cfg(windows)]
fn parse_windows_voices_json(value: &[u8]) -> Result<Vec<TtsVoice>, serde_json::Error> {
    let value = serde_json::from_slice::<serde_json::Value>(value)?;
    if let Some(items) = value.as_array() {
        return items
            .iter()
            .cloned()
            .map(serde_json::from_value::<TtsVoice>)
            .collect();
    }
    serde_json::from_value::<TtsVoice>(value).map(|voice| vec![voice])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(windows)]
    #[test]
    fn windows_voice_json_accepts_array() {
        let voices = parse_windows_voices_json(
            br#"[{"id":"Microsoft Zira Desktop","name":"Microsoft Zira Desktop","language":"en-US"}]"#,
        )
        .unwrap();

        assert_eq!(
            voices,
            vec![TtsVoice {
                id: "Microsoft Zira Desktop".into(),
                name: "Microsoft Zira Desktop".into(),
                language: "en-US".into(),
            }]
        );
    }

    #[test]
    fn speech_dispatcher_volume_maps_ui_range() {
        assert_eq!(speech_dispatcher_volume(0), -100);
        assert_eq!(speech_dispatcher_volume(50), 0);
        assert_eq!(speech_dispatcher_volume(100), 100);
        assert_eq!(speech_dispatcher_volume(255), 100);
    }
}
