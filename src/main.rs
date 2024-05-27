// Copyright 2024 Metaa4245
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//     http://www.apache.org/licenses/LICENSE-2.0
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::{fs::File, ptr::null_mut, sync::Arc};

use nwd::NwgUi;
use nwg::{CheckBoxState, NativeUi};
use rustysynth::{MidiFile, MidiFileSequencer, SoundFont, Synthesizer, SynthesizerSettings};
use winapi::um::winuser::{MessageBoxW, MB_OK};

fn checkbox_state_as_bool(state: CheckBoxState) -> bool {
    match state {
        CheckBoxState::Checked => true,
        CheckBoxState::Unchecked => false,
        CheckBoxState::Indeterminate => false,
    }
}

#[derive(Default, NwgUi)]
pub struct FireSynth {
    #[nwg_control(size: (300, 300), title: "FireSynth", flags: "WINDOW|VISIBLE|RESIZABLE")]
    #[nwg_events(OnWindowClose: [FireSynth::close], OnInit: [FireSynth::window_init])]
    window: nwg::Window,

    #[nwg_layout(parent: window, spacing: 2, min_size: [150, 140])]
    grid: nwg::GridLayout,

    #[nwg_control(text: "Select MIDI")]
    #[nwg_layout_item(layout: grid, col: 0, row: 0, col_span: 1, row_span: 2)]
    #[nwg_events(OnButtonClick: [FireSynth::midi_select])]
    midi_button: nwg::Button,

    #[nwg_control(text: "Select SoundFont")]
    #[nwg_layout_item(layout: grid, col: 0, row: 2, col_span: 1, row_span: 2)]
    #[nwg_events(OnButtonClick: [FireSynth::sf_select])]
    sf_button: nwg::Button,

    #[nwg_control(text: "Select Dest")]
    #[nwg_layout_item(layout: grid, col: 0, row: 4, col_span: 1, row_span: 2)]
    #[nwg_events(OnButtonClick: [FireSynth::output_select])]
    output_button: nwg::Button,

    #[nwg_control(text: "Render")]
    #[nwg_layout_item(layout: grid, col: 0, row: 6, col_span: 1, row_span: 2)]
    #[nwg_events(OnButtonClick: [FireSynth::render])]
    render_button: nwg::Button,

    #[nwg_control(text: "MIDI Path")]
    #[nwg_layout_item(layout: grid, col: 1, row: 0, col_span: 1)]
    midi_label: nwg::Label,

    #[nwg_control(readonly: true)]
    #[nwg_layout_item(layout: grid, col: 1, row: 1, col_span: 1)]
    midi_path: nwg::TextInput,

    #[nwg_control(text: "SoundFont Path")]
    #[nwg_layout_item(layout: grid, col: 1, row: 2, col_span: 1)]
    sf_label: nwg::Label,

    #[nwg_control(readonly: true)]
    #[nwg_layout_item(layout: grid, col: 1, row: 3, col_span: 1)]
    sf_path: nwg::TextInput,

    #[nwg_control(text: "Output Path")]
    #[nwg_layout_item(layout: grid, col: 1, row: 4, col_span: 1)]
    output_label: nwg::Label,

    #[nwg_control(readonly: true)]
    #[nwg_layout_item(layout: grid, col: 1, row: 5, col_span: 1)]
    output_path: nwg::TextInput,

    #[nwg_control(text: "Sample Rate")]
    #[nwg_layout_item(layout: grid, col: 1, row: 6, col_span: 1)]
    rate_label: nwg::Label,

    #[nwg_control(text: "44100")]
    #[nwg_layout_item(layout: grid, col: 1, row: 7, col_span: 1)]
    sample_rate: nwg::TextInput,

    #[nwg_control(text: "Reverb and chorus")]
    #[nwg_layout_item(layout: grid, col: 0, row: 8, col_span: 2)]
    reverb: nwg::CheckBox,

    #[nwg_control(readonly: true)]
    #[nwg_layout_item(layout: grid, col: 0, row: 9, col_span: 2)]
    status: nwg::TextInput,

    #[nwg_resource(title: "Open MIDI", action: nwg::FileDialogAction::Open, filters: "MIDI(*.mid;*.midi)")]
    midi_dialog: nwg::FileDialog,

    #[nwg_resource(title: "Open SoundFont", action: nwg::FileDialogAction::Open, filters: "SoundFont(*.sf;*.sf2;*.sf3)")]
    sf_dialog: nwg::FileDialog,

    #[nwg_resource(title: "Save File", action: nwg::FileDialogAction::Save, filters: "WAV(*.wav)")]
    save_file_dialog: nwg::FileDialog,
}

impl FireSynth {
    fn window_init(&self) {
        std::panic::set_hook(Box::new(|info| {
            let backtrace = std::backtrace::Backtrace::force_capture();

            let mut panic_str: Vec<u16> = format!("panic info: {info}\n backtrace: {backtrace}")
                .as_str()
                .encode_utf16()
                .collect();
            panic_str.push(0);
            let panic_str = panic_str.as_ptr();

            let mut title_str: Vec<u16> = "Panic".encode_utf16().collect();
            title_str.push(0);
            let title_str = title_str.as_ptr();

            // safety: calls MessageBoxW
            unsafe {
                MessageBoxW(null_mut(), panic_str, title_str, MB_OK);
            }
        }));
    }

    fn midi_select(&self) {
        if self.midi_dialog.run(Some(&self.window)) {
            self.midi_path.set_text("");
            if let Ok(dir) = self.midi_dialog.get_selected_item() {
                self.midi_path.set_text(&dir.into_string().unwrap());
            }
        }
    }

    fn sf_select(&self) {
        if self.sf_dialog.run(Some(&self.window)) {
            self.sf_path.set_text("");
            if let Ok(dir) = self.sf_dialog.get_selected_item() {
                self.sf_path.set_text(&dir.into_string().unwrap());
            }
        }
    }

    fn output_select(&self) {
        if self.save_file_dialog.run(Some(&self.window)) {
            self.output_path.set_text("");
            if let Ok(dir) = self.save_file_dialog.get_selected_item() {
                self.output_path.set_text(&dir.into_string().unwrap());
            }
        }
    }

    fn render(&self) {
        let sample_rate: i32 = self.sample_rate.text().parse().unwrap();

        let mut sf = File::open(self.sf_path.text()).unwrap();
        let sound_font = Arc::new(SoundFont::new(&mut sf).unwrap());

        let mut midi = File::open(self.midi_path.text()).unwrap();
        let midi_file = Arc::new(MidiFile::new(&mut midi).unwrap());

        let mut settings = SynthesizerSettings::new(sample_rate);
        settings.enable_reverb_and_chorus = checkbox_state_as_bool(self.reverb.check_state());
        let synthesizer = Synthesizer::new(&sound_font, &settings).unwrap();
        let mut sequencer = MidiFileSequencer::new(synthesizer);

        sequencer.play(&midi_file, false);

        let sample_count = (sample_rate as f64 * midi_file.get_length()) as usize;
        let mut left: Vec<f32> = vec![0_f32; sample_count];
        let mut right: Vec<f32> = vec![0_f32; sample_count];

        sequencer.render(&mut left[..], &mut right[..]);

        let spec = hound::WavSpec {
            channels: 2,
            sample_rate: sample_rate as u32,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        };
        let mut writer = hound::WavWriter::create(self.output_path.text(), spec).unwrap();

        for sample in left.iter().zip(right.iter()) {
            writer.write_sample(*sample.0).unwrap(); // left
            writer.write_sample(*sample.1).unwrap(); // right
        }

        self.status.set_text("Done");
    }

    fn close(&self) {
        nwg::stop_thread_dispatch();
    }
}

fn main() {
    nwg::init().expect("initializing native windows GUI failed");
    nwg::Font::set_global_family("Segoe UI").expect("setting font failed");
    let _app = FireSynth::build_ui(Default::default()).expect("building UI failed");
    nwg::dispatch_thread_events();
}
