use eframe::egui::text::{CCursorRange, LayoutJob};
use eframe::egui::{self, TextBuffer};
use eframe::egui::{Color32, Stroke, TextFormat};
use std::path::PathBuf;
use std::rc::Rc;
use std::{env, fs};

mod note;
use crate::note::{MarkdownString, MarkdownType, Note};

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

fn render_markdown(strings: Vec<MarkdownString>) -> LayoutJob {
    let mut job = LayoutJob::default();

    for s in strings {
        match s.mdtype {
            MarkdownType::Heading => {
                draw_bold(&mut job, &s.text);
            }
            MarkdownType::Paragraph => {
                draw_normal(&mut job, &s.text);
            }
            MarkdownType::Bold => {
                draw_bold(&mut job, &s.text);
            }
            MarkdownType::Italic => {
                draw_italic(&mut job, &s.text);
            }
            MarkdownType::Link => {
                draw_link(&mut job, &s.text);
            }
            _ => {}
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

/*pub trait AsAny {
    fn as_any(&self) -> &dyn Any;
    fn as_mut_any(&mut self) -> &mut dyn Any;
}

impl<T: Any> AsAny for dyn egui::TextBuffer {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_mut_any(&mut self) -> &mut dyn Any {
        self
    }
}*/

impl eframe::App for NoteRs {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(self.path.display().to_string());
            let markdown = self.note.markdown().clone();
            egui::ScrollArea::vertical().show(ui, |ui| {
                let mut layouter = |ui: &egui::Ui, buf: &dyn TextBuffer, _wrap_width: f32| {
                    // TODO: figure out the wacky AsAny downcast and use it here so the cursor stops flickering
                    let job = render_markdown(markdown.clone());

                    ui.fonts_mut(|f| f.layout_job(job))
                };
                let editor = egui::TextEdit::multiline(&mut self.note)
                    .desired_width(f32::INFINITY)
                    .desired_rows((ctx.content_rect().height() / 16f32) as usize)
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

                        let node = self.note.get_node(idx);
                        match node.mdtype {
                            MarkdownType::Link => {
                                self.open_file(node.text[2..].to_string());
                            }
                            _ => {}
                        }
                    }
                }
            });
            if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::S)) {
                self.save_file();
            }
            if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::T)) {
                // TODO: translate and toggle
                let path = self.note.path(self.cursor_range.primary.index);
                self.note.toggle(path.as_slice());
                self.note.refresh();
            }
        });
    }
}
