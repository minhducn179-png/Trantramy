use rodio::{Decoder, OutputStream, OutputStreamBuilder, Sink, Source};
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::Read,
    sync::{Arc, Mutex},
    time::Duration,
};
pub const FREQUENCIES: [f32; 10] = [
    32., 64., 125., 250., 500., 1000., 2000., 4000., 8000., 16000.,
];
#[derive(Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct EqSettings {
    pub enabled: bool,
    pub limiter: bool,
    pub preamp: f32,
    pub gains: [f32; 10],
}
impl Default for EqSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            limiter: true,
            preamp: 0.,
            gains: [0.; 10],
        }
    }
}
#[derive(Clone, Copy)]
struct Biquad {
    b: [f32; 3],
    a: [f32; 2],
    z: [f32; 2],
}
impl Default for Biquad {
    fn default() -> Self {
        Self {
            b: [1., 0., 0.],
            a: [0., 0.],
            z: [0., 0.],
        }
    }
}
impl Biquad {
    fn peaking(frequency: f32, rate: u32, db: f32) -> Self {
        let w = 2. * std::f32::consts::PI * frequency.min(rate as f32 * 0.45) / rate as f32;
        let a = 10f32.powf(db / 40.);
        let alpha = w.sin() / 2.;
        let base = 1. + alpha / a;
        Self {
            b: [
                (1. + alpha * a) / base,
                -2. * w.cos() / base,
                (1. - alpha * a) / base,
            ],
            a: [-2. * w.cos() / base, (1. - alpha / a) / base],
            z: [0., 0.],
        }
    }
    fn sample(&mut self, x: f32) -> f32 {
        let y = self.b[0] * x + self.z[0];
        self.z[0] = self.b[1] * x - self.a[0] * y + self.z[1];
        self.z[1] = self.b[2] * x - self.a[1] * y;
        y
    }
}
pub struct Equalized<S> {
    source: S,
    settings: Arc<Mutex<EqSettings>>,
    snapshot: EqSettings,
    filters: Vec<[Biquad; 10]>,
    sample: usize,
    waveform: Arc<Mutex<[f32; 32]>>,
    peak: f32,
}
impl<S: Source> Equalized<S> {
    pub fn new(
        source: S,
        settings: Arc<Mutex<EqSettings>>,
        waveform: Arc<Mutex<[f32; 32]>>,
    ) -> Self {
        let snapshot = settings.lock().map(|v| v.clone()).unwrap_or_default();
        let filters = Self::filters(source.channels(), source.sample_rate(), &snapshot);
        Self {
            source,
            settings,
            snapshot,
            filters,
            sample: 0,
            waveform,
            peak: 0.,
        }
    }
    fn filters(channels: u16, rate: u32, settings: &EqSettings) -> Vec<[Biquad; 10]> {
        vec![
            std::array::from_fn(|i| Biquad::peaking(
                FREQUENCIES[i],
                rate,
                settings.gains[i].clamp(-12., 12.)
            ));
            channels as usize
        ]
    }
}
impl<S: Source> Iterator for Equalized<S> {
    type Item = f32;
    fn next(&mut self) -> Option<f32> {
        let mut x = self.source.next()?;
        if self.sample.is_multiple_of(2048) {
            if let Ok(next) = self.settings.try_lock()
                && *next != self.snapshot
            {
                self.snapshot = next.clone();
                self.filters = Self::filters(
                    self.source.channels(),
                    self.source.sample_rate(),
                    &self.snapshot,
                );
            }
            if let Ok(mut wave) = self.waveform.try_lock() {
                wave.rotate_left(1);
                wave[31] = self.peak;
            }
            self.peak = 0.;
        }
        if self.snapshot.enabled {
            x *= 10f32.powf(self.snapshot.preamp.clamp(-12., 12.) / 20.);
            let channel = self.sample % self.filters.len();
            for filter in &mut self.filters[channel] {
                x = filter.sample(x);
            }
            if self.snapshot.limiter {
                x = x.clamp(-1., 1.);
            }
        }
        self.sample += 1;
        self.peak = self.peak.max(x.abs());
        Some(x)
    }
}
impl<S: Source> Source for Equalized<S> {
    fn current_span_len(&self) -> Option<usize> {
        self.source.current_span_len()
    }
    fn channels(&self) -> u16 {
        self.source.channels()
    }
    fn sample_rate(&self) -> u32 {
        self.source.sample_rate()
    }
    fn total_duration(&self) -> Option<Duration> {
        self.source.total_duration()
    }
    fn try_seek(&mut self, pos: Duration) -> Result<(), rodio::source::SeekError> {
        self.source.try_seek(pos)?;
        self.filters = Self::filters(
            self.source.channels(),
            self.source.sample_rate(),
            &self.snapshot,
        );
        Ok(())
    }
}
pub struct Prepared {
    pub file: File,
    pub temporary: Option<tempfile::NamedTempFile>,
}
pub fn prepare(path: &str) -> Result<Prepared, String> {
    if path.starts_with("https://") || path.starts_with("http://") {
        let response = ureq::AgentBuilder::new()
            .timeout(Duration::from_secs(120))
            .build()
            .get(path)
            .call()
            .map_err(|e| e.to_string())?;
        let mut file = tempfile::NamedTempFile::new().map_err(|e| e.to_string())?;
        let length = std::io::copy(
            &mut response.into_reader().take(512 * 1024 * 1024 + 1),
            &mut file,
        )
        .map_err(|e| e.to_string())?;
        if length > 512 * 1024 * 1024 {
            return Err(
                "Nguồn âm thanh vượt giới hạn 512 MB. Tải bản nhạc về thư viện trước.".into(),
            );
        }
        let opened = File::open(file.path()).map_err(|e| e.to_string())?;
        Ok(Prepared {
            file: opened,
            temporary: Some(file),
        })
    } else {
        Ok(Prepared {
            file: File::open(path).map_err(|e| format!("Không mở được bản nhạc: {e}"))?,
            temporary: None,
        })
    }
}
pub struct Audio {
    pub sink: Option<Sink>,
    stream: Option<OutputStream>,
    temporary: Option<tempfile::NamedTempFile>,
    pub eq: Arc<Mutex<EqSettings>>,
    pub waveform: Arc<Mutex<[f32; 32]>>,
    pub duration: f64,
}
impl Default for Audio {
    fn default() -> Self {
        Self {
            sink: None,
            stream: None,
            temporary: None,
            eq: Arc::new(Mutex::new(EqSettings::default())),
            waveform: Arc::new(Mutex::new([0.; 32])),
            duration: 0.,
        }
    }
}
impl Audio {
    pub fn load(&mut self, prepared: Prepared, volume: f32) -> Result<(), String> {
        if self.stream.is_none() {
            self.stream = Some(
                OutputStreamBuilder::open_default_stream()
                    .map_err(|e| format!("Không mở được thiết bị âm thanh: {e}"))?,
            );
        }
        let decoder = Decoder::try_from(prepared.file)
            .map_err(|e| format!("Không giải mã được âm thanh: {e}"))?;
        self.duration = decoder.total_duration().map_or(0., |d| d.as_secs_f64());
        self.stop();
        self.temporary = prepared.temporary;
        let sink = Sink::connect_new(self.stream.as_ref().unwrap().mixer());
        sink.set_volume(volume.clamp(0., 1.));
        sink.append(Equalized::new(
            decoder,
            self.eq.clone(),
            self.waveform.clone(),
        ));
        self.sink = Some(sink);
        Ok(())
    }
    pub fn stop(&mut self) {
        if let Some(sink) = self.sink.take() {
            sink.stop();
        }
        self.temporary = None;
    }
    pub fn position(&self) -> f64 {
        self.sink.as_ref().map_or(0., |s| s.get_pos().as_secs_f64())
    }
    pub fn seek(&self, seconds: f64) -> Result<(), String> {
        if !seconds.is_finite() || seconds < 0. {
            return Err("Vị trí tua không hợp lệ.".into());
        }
        if let Some(s) = &self.sink {
            s.try_seek(Duration::from_secs_f64(seconds))
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn peaking_filters_change_frequency_response_and_keep_flat_audio() {
        fn rms(db: f32, freq: f32) -> f32 {
            let mut f = Biquad::peaking(1000., 48000, db);
            let mut sum = 0.;
            for i in 0..48000 {
                let x = (2. * std::f32::consts::PI * freq * i as f32 / 48000.).sin() * 0.1;
                let y = f.sample(x);
                if i > 24000 {
                    sum += y * y;
                }
            }
            (sum / 23999.).sqrt()
        }
        let flat = rms(0., 1000.);
        let boosted = rms(6., 1000.);
        assert!((boosted / flat - 10f32.powf(6. / 20.)).abs() < 0.02);
        assert!((rms(6., 100.) / rms(0., 100.) - 1.).abs() < 0.03);
        let source = rodio::buffer::SamplesBuffer::new(1, 48000, vec![0.25f32; 100]);
        let mut eq = Equalized::new(
            source,
            Arc::new(Mutex::new(EqSettings::default())),
            Arc::new(Mutex::new([0.; 32])),
        );
        assert!((eq.next().unwrap() - 0.25).abs() < 0.0001);
    }
}
