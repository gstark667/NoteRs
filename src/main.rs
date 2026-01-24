use eframe::egui;
use eframe::egui::text::{CCursorRange, LayoutJob};
use eframe::egui::{Color32, Stroke, TextFormat};
use regex::Regex;
use std::path::PathBuf;
use std::{env, fs};

mod note;
use crate::note::Note;

fn main() {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "NoteRs",
        native_options,
        Box::new(|cc| Ok(Box::new(NoteRs::new(cc)))),
    );
}

#[derive(Default)]
struct NoteRs {
    root: PathBuf,
    path: PathBuf,
    //text: String,
    cursor_range: CCursorRange,
    note: Note,
    link_spans: Vec<(std::ops::Range<usize>, String)>,
}

fn draw_normal(job: &mut LayoutJob, text: &String) {
    job.append(
        text,
        0.0,
        TextFormat {
            color: Color32::from_rgb(120, 120, 120),
            ..Default::default()
        },
    );
}

fn draw_bold(job: &mut LayoutJob, text: &String) {
    job.append(
        text,
        0.0,
        TextFormat {
            color: Color32::from_rgb(255, 255, 255),
            ..Default::default() // todo: bold
        },
    );
}

fn draw_italic(job: &mut LayoutJob, text: &String) {
    job.append(
        text,
        0.0,
        TextFormat {
            color: Color32::from_rgb(180, 180, 180),
            italics: true,
            ..Default::default()
        },
    );
}

fn draw_link(job: &mut LayoutJob, text: &String) {
    job.append(
        &text,
        0.0,
        TextFormat {
            color: Color32::from_rgb(80, 140, 255),
            underline: Stroke::new(1.0, Color32::from_rgb(80, 140, 255)),
            ..Default::default()
        },
    );
}

fn render_markdown(
    text: &str,
    link_spans: &mut Vec<(std::ops::Range<usize>, String)>,
) -> LayoutJob {
    link_spans.clear();

    let regexes: [(Regex, &str, fn(&mut LayoutJob, &String)); 4] = [
        (
            Regex::new(r"(?m)^(#+)([^\n]+)$").unwrap(),
            "heading",
            draw_bold,
        ),
        (Regex::new(r"\*\*[^\*]+\*\*").unwrap(), "bold", draw_bold),
        (Regex::new(r"_[^_]+_").unwrap(), "italic", draw_italic),
        (
            Regex::new(r"@@([\\/A-Za-z0-9_-]+)").unwrap(),
            "link",
            draw_link,
        ),
    ];

    let mut job = LayoutJob::default();
    let mut lines = text.split('\n').peekable();
    let mut offset = 0;

    while let Some(line) = lines.next() {
        let is_last = lines.peek().is_none();
        let mut t = String::from(line);

        let mut first_match: Option<((usize, usize), &str, fn(&mut LayoutJob, &String))> = None;
        let mut rerun = true;
        while rerun {
            rerun = false;
            first_match = None;
            for r in &regexes {
                if let Some(mat) = r.0.find(t.as_str()) {
                    let range = mat.range();

                    // give up early if there was a match before this
                    if let Some(first) = first_match
                        && first.0.0 < range.start
                    {
                        continue;
                    }

                    first_match = Some(((range.start, range.end), r.1, r.2));
                }
            }

            if let Some(first) = first_match {
                if first.0.0 > 0 {
                    let head: String = t[..first.0.0].to_string();
                    draw_normal(&mut job, &head);
                }
                let data: String = t[first.0.0..first.0.1].to_string();
                first.2(&mut job, &data);
                t = t[first.0.1..].to_string();

                // this is also a terrible solution, but works for now
                if first.1 == "link" {
                    link_spans.push((
                        std::ops::Range {
                            start: first.0.0 + offset,
                            end: first.0.1 + offset,
                        },
                        data.chars().skip(2).take(data.len() - 2).collect(),
                    ));
                }
                offset += data.len();
                rerun = true;
            }
        }
        if t.len() > 0 {
            draw_normal(&mut job, &t);
            offset += t.len();
        }

        if !is_last {
            job.append(
                "\n",
                0.0,
                TextFormat {
                    color: Color32::from_rgb(255, 255, 255),
                    ..Default::default()
                },
            );
            offset += 1;
        }
    }
    return job;
}

impl NoteRs {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        let mut new_one = Self::default();
        match env::home_dir() {
            Some(path) => {
                new_one.root = path;
                new_one.root.push("NoteRs");
            }
            None => println!("Impossible to get your home dir!"),
        }
        new_one.open_file("index.md".to_string());
        return new_one;
    }

    fn open_file(&mut self, path: String) {
        self.path = self.root.clone();

        let binding = PathBuf::from(path);
        let mut iter = binding.components().peekable();
        while let Some(item) = iter.next() {
            let is_last = iter.peek().is_none();
            self.path.push(item);

            if is_last {
                if self.path.exists() {
                    if self.path.is_dir() {
                        println!("exists already, add index.md");
                        self.path.push("index.md");
                    } else {
                        println!("path is a file");
                    }
                } else {
                    println!("not a folder, add .md");
                    self.path.set_extension("md");
                }
            } else {
                if let Err(e) = fs::create_dir_all(self.path.as_path()) {
                    eprintln!("Failed to create directory: {}", e);
                }
            }
        }

        println!("opening {}", self.path.display());
        if self.path.exists() {
            match fs::read_to_string(self.path.as_path()) {
                Ok(text) => self.note = Note::new(text),
                Err(e) => println!("error opening file: {e:?}"),
            }

            //.expect("Should have been able to read the file");
        } else {
            self.note = Note::default();
        }
    }

    fn save_file(&self) {
        let text = self.note.full();
        println!("Writing {}: {}", self.path.display(), text);
        fs::write(self.path.as_path(), text.as_bytes());
    }
}

impl eframe::App for NoteRs {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut layouter = |ui: &egui::Ui, buf: &dyn egui::TextBuffer, _wrap_width: f32| {
                let job = render_markdown(buf.as_str(), &mut self.link_spans);

                ui.fonts_mut(|f| f.layout_job(job))
            };

            ui.heading(self.path.display().to_string());
            let editor = egui::TextEdit::multiline(&mut self.note)
                .desired_width(f32::INFINITY)
                .desired_rows(30)
                .layouter(&mut layouter)
                .id(egui::Id::new("editor"))
                .show(ui);

            let response = editor.response;
            let galley = editor.galley;

            if let Some(cursor_range) = editor.cursor_range {
                if self.cursor_range.primary.index != cursor_range.primary.index
                    || self.cursor_range.secondary.index != cursor_range.primary.index
                {
                    println!("cursor moved: {:?}", cursor_range);
                }
                self.cursor_range = cursor_range;
            }

            if response.clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    let local_pos = pos - response.rect.min;
                    let cursor = galley.cursor_from_pos(local_pos);
                    let idx = cursor.index;

                    let mut to_open: Option<String> = None;
                    for (range, keyword) in &self.link_spans {
                        if range.contains(&idx) {
                            to_open = Some(keyword.to_string());
                            break;
                        }
                    }

                    if let Some(path) = to_open {
                        self.open_file(path)
                    }
                }
            }
            if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::S)) {
                self.save_file();
            }
            if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::T)) {
                // TODO: translate and toggle
                self.note.refresh();
            }
        });
    }
}
