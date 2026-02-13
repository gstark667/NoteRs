use cssparser_color::Color;
use eframe::egui::text::{CCursor, CCursorRange, LayoutJob};
use eframe::egui::text_edit::TextEditState;
use eframe::egui::{self, TextBuffer};
use eframe::egui::{Color32, CursorIcon, FontFamily, FontId, Stroke, TextFormat, Visuals};
use std::collections::HashMap;
use std::path::PathBuf;
use std::{env, fs};

mod note;
use crate::note::{MarkdownStr, MarkdownType, Note, highlight_parse};

fn main() {
    println!("{:?}", linux_theme::gtk::current::current());
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
    cursor_range: CCursorRange,
    note: Note,
    nav_history: Vec<String>,
    nav_forward: Vec<String>,
    bg_color: Color32,
    fg_color: Color32,
}

fn draw_normal(job: &mut LayoutJob, text: &str) {
    job.append(
        text,
        0.0,
        TextFormat {
            color: Color32::from_rgb(180, 180, 180),
            ..Default::default()
        },
    );
}

fn draw_bold(job: &mut LayoutJob, text: &str) {
    job.append(
        text,
        0.0,
        TextFormat {
            color: Color32::from_rgb(255, 255, 255),
            ..Default::default() // todo: bold
        },
    );
}

fn draw_italic(job: &mut LayoutJob, text: &str) {
    job.append(
        text,
        0.0,
        TextFormat {
            color: Color32::from_rgb(200, 200, 200),
            italics: true,
            ..Default::default()
        },
    );
}

fn draw_monospace(job: &mut LayoutJob, text: &str) {
    job.append(
        text,
        0.0,
        TextFormat {
            font_id: FontId {
                size: 12.0,
                family: FontFamily::Monospace,
            },
            color: Color32::from_rgb(200, 200, 200),
            ..Default::default()
        },
    );
}

fn draw_heading(job: &mut LayoutJob, text: &str, level: usize) {
    job.append(
        text,
        0.0,
        TextFormat {
            font_id: FontId {
                size: match level {
                    1 => 32.0,
                    2 => 24.0,
                    _ => 16.0,
                },
                family: FontFamily::Proportional,
            },
            color: Color32::from_rgb(255, 255, 255),
            line_height: Some(match level {
                1 => 36.0,
                2 => 28.0,
                _ => 20.0,
            }),
            ..Default::default()
        },
    );
}

fn draw_link(job: &mut LayoutJob, text: &str) {
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

fn render_markdown(strings: Vec<MarkdownStr<'_>>) -> LayoutJob {
    let mut job = LayoutJob::default();

    for s in strings {
        match s.mdtype {
            MarkdownType::Heading1 => {
                draw_heading(&mut job, &s.text, 1);
            }
            MarkdownType::Heading2 => {
                draw_heading(&mut job, &s.text, 2);
            }
            MarkdownType::Heading3 => {
                draw_heading(&mut job, &s.text, 3);
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
            MarkdownType::Monospace => {
                draw_monospace(&mut job, &s.text);
            }
            MarkdownType::Code => {
                draw_monospace(&mut job, &s.text);
            }
            _ => {}
        }
    }
    return job;
}

fn make_color32(inp: &Color) -> Color32 {
    match inp {
        Color::Rgba(rgba) => Color32::from_rgb(rgba.red, rgba.green, rgba.blue),
        _ => Color32::TRANSPARENT,
    }
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

        // TODO: figure out a qt way to do this too
        let colors = linux_theme::gtk::current::current().0;
        //new_one.bg_color = make_color32(colors.get("window_bg_color").unwrap());
        // TODO: pull these in using a qt lib/detect GTK and use other lib
        new_one.bg_color = Color32::from_rgb(30, 32, 48);
        new_one.fg_color = Color32::from_rgb(202, 211, 248);
        new_one.open_file("index.md".to_string());

        let mut visuals = Visuals::dark();
        visuals.window_fill = new_one.bg_color;
        visuals.panel_fill = new_one.bg_color;
        cc.egui_ctx.set_visuals(visuals);

        println!("{:?}", new_one.bg_color);

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
                Ok(text) => {
                    println!("`\n{}\n`", text);
                    self.note = Note::new(text)
                }
                Err(e) => println!("error opening file: {e:?}"),
            }
        } else {
            self.note = Note::default();
        }
    }

    fn save_file(&mut self) {
        let text = self.note.full();
        println!("Writing {}: {}", self.path.display(), text);
        fs::write(self.path.as_path(), text.as_bytes());
    }
}

impl eframe::App for NoteRs {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let text_edit_id = ui.make_persistent_id("editor");
            ui.heading(self.path.display().to_string());
            egui::ScrollArea::vertical().show(ui, |ui| {
                let mut layouter = |ui: &egui::Ui, buf: &dyn TextBuffer, _wrap_width: f32| {
                    // TODO: consider how to make this faster than just reparsing the whole thing
                    //let new_note = Note::new(buf.as_str().to_string());
                    //let job = render_markdown(new_note.markdown());
                    let job = render_markdown(highlight_parse(buf.as_str()));

                    ui.fonts_mut(|f| f.layout_job(job))
                };
                let editor = egui::TextEdit::multiline(&mut self.note)
                    .desired_width(f32::INFINITY)
                    .desired_rows((ctx.content_rect().height() / 16f32) as usize)
                    .layouter(&mut layouter)
                    .id(text_edit_id)
                    .show(ui);
                let response = editor.response;
                let galley = editor.galley;
                let painter = ui.painter();

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
                                self.nav_history
                                    .push(self.path.to_str().unwrap().to_string());
                                self.nav_forward.clear();
                                self.open_file(node.text[2..].to_string());
                            }
                            _ => {}
                        }
                    }
                } else {
                    // change the cursor icon when moving the mouse
                    if let Some(p) = ctx.input_mut(|i| i.pointer.hover_pos()) {
                        let local_pos = p - response.rect.min;
                        let cursor = galley.cursor_from_pos(local_pos);
                        let idx = cursor.index;
                        let node = self.note.get_node(idx);
                        match node.mdtype {
                            MarkdownType::Link => {
                                ctx.output_mut(|out| out.cursor_icon = CursorIcon::PointingHand)
                            }
                            _ => {}
                        }
                    }

                    let mut index = 0;
                    for item in self.note.markdown() {
                        index += item.text.len();
                        match item.mdtype {
                            MarkdownType::Heading1
                            | MarkdownType::Heading2
                            | MarkdownType::Heading3 => {
                                painter.text(
                                    galley.pos_from_cursor(CCursor::new(index)).min,
                                    egui::Align2::LEFT_TOP,
                                    if item.expanded { "V" } else { ">" },
                                    egui::FontId::default(),
                                    ui.visuals().text_color(),
                                );
                            }
                            _ => {}
                        }
                    }
                }

                if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::S)) {
                    self.save_file();
                }
                if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::T)) {
                    // TODO: translate and toggle
                    let path = self.note.path(self.cursor_range.primary.index);
                    let mut global_cursor = (
                        self.note.translate(self.cursor_range.primary.index),
                        self.note.translate(self.cursor_range.secondary.index),
                    );
                    self.note.toggle(path.as_slice());
                    self.note.refresh();
                    global_cursor.0 = self.note.inv_translate(global_cursor.0);
                    global_cursor.1 = self.note.inv_translate(global_cursor.1);

                    println!("updating cursor to: {:?}", editor.cursor_range);

                    if let Some(mut state) = TextEditState::load(ui.ctx(), text_edit_id) {
                        // Move cursor to position 10
                        //let cursor = editor.cursor_range; //CCursorRange::one(egui::text::CCursor::new(10));
                        println!("really updating");
                        state.cursor.set_char_range(Some(CCursorRange::two(
                            egui::text::CCursor::new(global_cursor.0),
                            egui::text::CCursor::new(global_cursor.1),
                        )));
                        state.store(ui.ctx(), text_edit_id);
                    }
                }
                if ctx.input_mut(|i| i.consume_key(egui::Modifiers::ALT, egui::Key::ArrowLeft)) {
                    println!("Nav back");

                    match self.nav_history.pop() {
                        Some::<String>(s) => {
                            self.nav_forward
                                .push(self.path.to_str().unwrap().to_string());
                            self.open_file(s);
                        }
                        _ => {}
                    }
                }
                if ctx.input_mut(|i| i.consume_key(egui::Modifiers::ALT, egui::Key::ArrowRight)) {
                    println!("Nav forward {:?} {:?}", self.nav_history, self.nav_forward);

                    match self.nav_forward.pop() {
                        Some::<String>(s) => {
                            self.nav_history
                                .push(self.path.to_str().unwrap().to_string());
                            self.open_file(s);
                        }
                        _ => {}
                    }
                }
            });
        });
    }
}
