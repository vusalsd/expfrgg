use rodio::{Source, Sink};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    terminal::{disable_raw_mode, enable_raw_mode},
};

struct WaveTableOscillator {
    sample_rate: u32,
    wave_table: Vec<f32>,
    index: f32,
    index_increment: f32,
    frequency: Arc<Mutex<f32>>,
}

impl WaveTableOscillator {
    fn new(sample_rate: u32, wave_table: Vec<f32>) -> WaveTableOscillator {
        WaveTableOscillator {
            sample_rate,
            wave_table,
            index: 0.0,
            index_increment: 0.0,
            frequency: Arc::new(Mutex::new(0.0)),
        }
    }

    fn get_frequency_control(&self) -> Arc<Mutex<f32>> {
        self.frequency.clone()
    }

    fn update_frequency(&mut self) {
        if let Ok(freq) = self.frequency.lock() {
            self.index_increment = *freq * self.wave_table.len() as f32 / self.sample_rate as f32;
        }
    }

    fn get_sample(&mut self) -> f32 {
        self.update_frequency();

        if self.index_increment == 0.0 {
            return 0.0;
        }

        let sample = self.lerp();
        self.index += self.index_increment;
        self.index %= self.wave_table.len() as f32;
        sample * 0.3
    }

    fn lerp(&self) -> f32 {
        let truncated_index = self.index as usize;
        let next_index = (truncated_index + 1) % self.wave_table.len();

        let next_index_weight = self.index - truncated_index as f32;
        let truncated_index_weight = 1.0 - next_index_weight;

        truncated_index_weight * self.wave_table[truncated_index]
            + next_index_weight * self.wave_table[next_index]
    }
}

impl Source for WaveTableOscillator {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        None
    }
}

impl Iterator for WaveTableOscillator {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.get_sample())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let wave_table_size = 64;
    let mut wave_table: Vec<f32> = Vec::with_capacity(wave_table_size);

    // Generate a sine wave table
    for i in 0..wave_table_size {
        wave_table.push((2.0 * std::f32::consts::PI * i as f32 / wave_table_size as f32).sin());
    }

    // Create oscillator
    let oscillator = WaveTableOscillator::new(44100, wave_table);
    let frequency_control = oscillator.get_frequency_control();

    // Set up audio output
    let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();
    sink.append(oscillator);

    // Complete keyboard frequency mapping
    let mut key_frequencies = HashMap::new();

    // Function keys and special keys
    key_frequencies.insert(KeyCode::F(1), 55.00);        // A1
    key_frequencies.insert(KeyCode::F(2), 58.27);        // A#1
    key_frequencies.insert(KeyCode::F(3), 61.74);        // B1
    key_frequencies.insert(KeyCode::F(4), 65.41);        // C2
    key_frequencies.insert(KeyCode::F(5), 69.30);        // C#2
    key_frequencies.insert(KeyCode::F(6), 73.42);        // D2
    key_frequencies.insert(KeyCode::F(7), 77.78);        // D#2
    key_frequencies.insert(KeyCode::F(8), 82.41);        // E2
    key_frequencies.insert(KeyCode::F(9), 87.31);        // F2
    key_frequencies.insert(KeyCode::F(10), 92.50);       // F#2
    key_frequencies.insert(KeyCode::F(11), 98.00);       // G2
    key_frequencies.insert(KeyCode::F(12), 103.83);      // G#2

    // Number row
    key_frequencies.insert(KeyCode::Char('1'), 1046.50); // C6
    key_frequencies.insert(KeyCode::Char('2'), 1108.73); // C#6
    key_frequencies.insert(KeyCode::Char('3'), 1174.66); // D6
    key_frequencies.insert(KeyCode::Char('4'), 1244.51); // D#6
    key_frequencies.insert(KeyCode::Char('5'), 1318.51); // E6
    key_frequencies.insert(KeyCode::Char('6'), 1396.91); // F6
    key_frequencies.insert(KeyCode::Char('7'), 1479.98); // F#6
    key_frequencies.insert(KeyCode::Char('8'), 1567.98); // G6
    key_frequencies.insert(KeyCode::Char('9'), 1661.22); // G#6
    key_frequencies.insert(KeyCode::Char('0'), 1760.00); // A6
    key_frequencies.insert(KeyCode::Char('-'), 1864.66); // A#6
    key_frequencies.insert(KeyCode::Char('='), 1975.53); // B6

    // Top row (QWERTY)
    key_frequencies.insert(KeyCode::Char('q'), 2093.00); // C7
    key_frequencies.insert(KeyCode::Char('w'), 523.25);  // C5
    key_frequencies.insert(KeyCode::Char('e'), 554.37);  // C#5
    key_frequencies.insert(KeyCode::Char('r'), 587.33);  // D5
    key_frequencies.insert(KeyCode::Char('t'), 622.25);  // D#5
    key_frequencies.insert(KeyCode::Char('y'), 659.25);  // E5
    key_frequencies.insert(KeyCode::Char('u'), 698.46);  // F5
    key_frequencies.insert(KeyCode::Char('i'), 739.99);  // F#5
    key_frequencies.insert(KeyCode::Char('o'), 783.99);  // G5
    key_frequencies.insert(KeyCode::Char('p'), 830.61);  // G#5
    key_frequencies.insert(KeyCode::Char('['), 880.00);  // A5
    key_frequencies.insert(KeyCode::Char(']'), 932.33);  // A#5
    key_frequencies.insert(KeyCode::Char('\\'), 987.77); // B5

    // Home row (ASDF)
    key_frequencies.insert(KeyCode::Char('a'), 261.63);  // C4
    key_frequencies.insert(KeyCode::Char('s'), 277.18);  // C#4
    key_frequencies.insert(KeyCode::Char('d'), 293.66);  // D4
    key_frequencies.insert(KeyCode::Char('f'), 311.13);  // D#4
    key_frequencies.insert(KeyCode::Char('g'), 329.63);  // E4
    key_frequencies.insert(KeyCode::Char('h'), 349.23);  // F4
    key_frequencies.insert(KeyCode::Char('j'), 369.99);  // F#4
    key_frequencies.insert(KeyCode::Char('k'), 392.00);  // G4
    key_frequencies.insert(KeyCode::Char('l'), 415.30);  // G#4
    key_frequencies.insert(KeyCode::Char(';'), 440.00);  // A4
    key_frequencies.insert(KeyCode::Char('\''), 466.16); // A#4

    // Bottom row (ZXCV)
    key_frequencies.insert(KeyCode::Char('z'), 130.81);  // C3
    key_frequencies.insert(KeyCode::Char('x'), 138.59);  // C#3
    key_frequencies.insert(KeyCode::Char('c'), 146.83);  // D3
    key_frequencies.insert(KeyCode::Char('v'), 155.56);  // D#3
    key_frequencies.insert(KeyCode::Char('b'), 164.81);  // E3
    key_frequencies.insert(KeyCode::Char('n'), 174.61);  // F3
    key_frequencies.insert(KeyCode::Char('m'), 185.00);  // F#3
    key_frequencies.insert(KeyCode::Char(','), 196.00);  // G3
    key_frequencies.insert(KeyCode::Char('.'), 207.65);  // G#3
    key_frequencies.insert(KeyCode::Char('/'), 220.00);  // A3

    // Special keys
    key_frequencies.insert(KeyCode::Char(' '), 110.00);     // A2
    key_frequencies.insert(KeyCode::Tab, 116.54);       // A#2
    key_frequencies.insert(KeyCode::Enter, 123.47);     // B2
    key_frequencies.insert(KeyCode::Backspace, 233.08); // A#3
    key_frequencies.insert(KeyCode::Delete, 246.94);    // B3
    key_frequencies.insert(KeyCode::Insert, 2217.46);   // C#7
    key_frequencies.insert(KeyCode::Home, 2349.32);     // D7
    key_frequencies.insert(KeyCode::End, 2489.02);      // D#7
    key_frequencies.insert(KeyCode::PageUp, 2637.02);   // E7
    key_frequencies.insert(KeyCode::PageDown, 2793.83); // F7

    // Arrow keys
    key_frequencies.insert(KeyCode::Up, 41.20);         // E1
    key_frequencies.insert(KeyCode::Down, 43.65);       // F1
    key_frequencies.insert(KeyCode::Left, 46.25);       // F#1
    key_frequencies.insert(KeyCode::Right, 49.00);      // G1

    // Additional punctuation
    key_frequencies.insert(KeyCode::Char('`'), 32.70);  // C1
    key_frequencies.insert(KeyCode::Char('~'), 34.65);  // C#1
    key_frequencies.insert(KeyCode::Char('!'), 36.71);  // D1
    key_frequencies.insert(KeyCode::Char('@'), 38.89);  // D#1
    key_frequencies.insert(KeyCode::Char('#'), 2959.96); // F#7
    key_frequencies.insert(KeyCode::Char('$'), 3135.96); // G7
    key_frequencies.insert(KeyCode::Char('%'), 3322.44); // G#7
    key_frequencies.insert(KeyCode::Char('^'), 3520.00); // A7
    key_frequencies.insert(KeyCode::Char('&'), 3729.31); // A#7
    key_frequencies.insert(KeyCode::Char('*'), 3951.07); // B7
    key_frequencies.insert(KeyCode::Char('('), 4186.01); // C8
    key_frequencies.insert(KeyCode::Char(')'), 4434.92); // C#8

    println!("Press ESC to exit");

    // Enable raw mode for immediate key detection
    enable_raw_mode()?;

    loop {
        if event::poll(Duration::from_millis(10))? {
            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                match code {
                    KeyCode::Esc => break,
                    key => {
                        if let Some(&frequency) = key_frequencies.get(&key) {
                            // Play the note
                            if let Ok(mut freq) = frequency_control.lock() {
                                *freq = frequency;
                            }
                        } else {
                            // For any unmapped key, assign a random frequency
                            if let Ok(mut freq) = frequency_control.lock() {
                                *freq = 200.0 + (std::ptr::addr_of!(key) as usize % 1000) as f32;
                            }
                        }
                    }
                }
            }
        }

        // Small delay to prevent excessive CPU usage
        thread::sleep(Duration::from_millis(1));
    }

    // Restore terminal
    disable_raw_mode()?;

    Ok(())
}