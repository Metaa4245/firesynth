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

use std::{fs::File, sync::Arc};

use nwd::NwgUi;
use nwg::NativeUi;
use rustysynth::{MidiFile, MidiFileSequencer, SoundFont, Synthesizer, SynthesizerSettings};

#[derive(Default, NwgUi)]
pub struct FireSynth {
    #[nwg_control(size: (300, 300), position: (300, 300), title: "FireSynth", flags: "WINDOW|VISIBLE")]
    #[nwg_events(OnWindowClose: [FireSynth::close])]
    window: nwg::Window,

    #[nwg_layout(parent: window, spacing: 2, min_size: [150, 140])]
    grid: nwg::GridLayout,

    #[nwg_control(text: "Select MIDI")]
    #[nwg_layout_item(layout: grid, col: 0, row: 0, col_span: 1)]
    #[nwg_events(OnButtonClick: [FireSynth::midi_select])]
    midi_button: nwg::Button,

    #[nwg_control(text: "Select SoundFont")]
    #[nwg_layout_item(layout: grid, col: 0, row: 1, col_span: 1)]
    #[nwg_events(OnButtonClick: [FireSynth::sf_select])]
    sf_button: nwg::Button,

    #[nwg_control(text: "Select Dest")]
    #[nwg_layout_item(layout: grid, col: 0, row: 2, col_span: 1)]
    #[nwg_events(OnButtonClick: [FireSynth::output_select])]
    output_button: nwg::Button,

    #[nwg_control(text: "Render")]
    #[nwg_layout_item(layout: grid, col: 0, row: 3, col_span: 1)]
    #[nwg_events(OnButtonClick: [FireSynth::render])]
    render_button: nwg::Button,

    #[nwg_control(readonly: true)]
    #[nwg_layout_item(layout: grid, col: 0, row: 4, col_span: 1)]
    midi_path: nwg::TextInput,

    #[nwg_control(readonly: true)]
    #[nwg_layout_item(layout: grid, col: 0, row: 5, col_span: 1)]
    sf_path: nwg::TextInput,

    #[nwg_control(readonly: true)]
    #[nwg_layout_item(layout: grid, col: 0, row: 6, col_span: 1)]
    output_path: nwg::TextInput,

    #[nwg_control(readonly: true)]
    #[nwg_layout_item(layout: grid, col: 0, row: 7, col_span: 1)]
    status: nwg::TextInput,

    #[nwg_resource(title: "Open File", action: nwg::FileDialogAction::Open)]
    open_file_dialog: nwg::FileDialog,

    #[nwg_resource(title: "Save File", action: nwg::FileDialogAction::Save)]
    save_file_dialog: nwg::FileDialog,
}

impl FireSynth {
    fn midi_select(&self) {
        if self.open_file_dialog.run(Some(&self.window)) {
            self.midi_path.set_text("");
            if let Ok(dir) = self.open_file_dialog.get_selected_item() {
                self.midi_path.set_text(&dir.into_string().unwrap());
            }
        }
    }

    fn sf_select(&self) {
        if self.open_file_dialog.run(Some(&self.window)) {
            self.sf_path.set_text("");
            if let Ok(dir) = self.open_file_dialog.get_selected_item() {
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
        let mut sf = File::open(self.sf_path.text()).unwrap();
        let sound_font = Arc::new(SoundFont::new(&mut sf).unwrap());

        let mut midi = File::open(self.midi_path.text()).unwrap();
        let midi_file = Arc::new(MidiFile::new(&mut midi).unwrap());

        let settings = SynthesizerSettings::new(44100);
        let synthesizer = Synthesizer::new(&sound_font, &settings).unwrap();
        let mut sequencer = MidiFileSequencer::new(synthesizer);

        sequencer.play(&midi_file, false);

        let sample_count = (settings.sample_rate as f64 * midi_file.get_length()) as usize;
        let mut left: Vec<f32> = vec![0_f32; sample_count];
        let mut right: Vec<f32> = vec![0_f32; sample_count];

        sequencer.render(&mut left[..], &mut right[..]);

        let spec = hound::WavSpec {
            channels: 2,
            sample_rate: 44100,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        };
        let mut writer = hound::WavWriter::create(self.output_path.text(), spec).unwrap();

        for sample in left.iter().zip(right.iter()) {
            let (sample_left, sample_right) = sample;
            writer.write_sample(*sample_left).unwrap();
            writer.write_sample(*sample_right).unwrap();
        }

        self.status.set_text("Done");
    }

    fn close(&self) {
        nwg::stop_thread_dispatch();
    }
}

fn main() {
    nwg::init().expect("initializing native windows GUI failed");
    nwg::Font::set_global_family("Arial").expect("setting font failed");
    let _app = FireSynth::build_ui(Default::default()).expect("building UI failed");
    nwg::dispatch_thread_events();
}
