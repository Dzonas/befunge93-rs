use befunge93_rs::Interpreter;
use rand::rngs::ThreadRng;
use std::io::Cursor;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    env_logger::init();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([300.0, 220.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Befunge93-rs",
        native_options,
        Box::new(|cc| Ok(Box::new(Befunge93App::new(cc)))),
    )
}

#[cfg(target_arch = "wasm32")]
fn main() {
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        let start_result = eframe::WebRunner::new()
            .start(
                "the_canvas_id",
                web_options,
                Box::new(|cc| Ok(Box::new(Befunge93App::new(cc)))),
            )
            .await;
        let loading_text = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id("loading_text"));

        if let Some(loading_text) = loading_text {
            match start_result {
                Ok(_) => {
                    loading_text.remove();
                }
                Err(e) => {
                    loading_text.set_inner_html(
                        "<p>The app has crashed. See the developer console for details.</p>",
                    );
                    panic!("Failed to start eframe: {e:?}");
                }
            }
        }
    });
}

struct Befunge93App {
    program: String,
    interpreter: Interpreter<Cursor<String>, Cursor<Vec<u8>>, ThreadRng>,
    running: bool,
}

impl Befunge93App {
    fn build_interpreter() -> Interpreter<Cursor<String>, Cursor<Vec<u8>>, ThreadRng> {
        let input = Cursor::new(String::new());
        let output = Cursor::new(Vec::new());
        let gen = rand::thread_rng();

        Interpreter::new(input, output, gen)
    }
    fn new(_: &eframe::CreationContext<'_>) -> Self {
        let interpreter = Self::build_interpreter();
        let program = String::new();
        let running = false;

        Befunge93App {
            program,
            interpreter,
            running,
        }
    }
}

impl eframe::App for Befunge93App {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        egui::SidePanel::left("left_panel").show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.heading("Input");
            });
            ui.add_sized(
                ui.available_size(),
                egui::TextEdit::multiline(self.interpreter.get_input_mut().get_mut())
                    .font(egui::TextStyle::Monospace),
            );
        });

        egui::SidePanel::right("right_panel").show(ctx, |ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                ui.heading("Stack")
            });
            ui.vertical_centered(|ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for value in self.interpreter.get_stack().iter().map(|x| x.to_string()) {
                        egui::Frame::none()
                            .fill(egui::Color32::DARK_GRAY)
                            .show(ui, |ui| {
                                ui.label(
                                    egui::RichText::new(value)
                                        .color(egui::Color32::WHITE)
                                        .size(20.0),
                                );
                            });
                    }
                });
            });
        });

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Controls");
            });

            ui.horizontal(|ui| {
                if ui.button("Load program").clicked() {
                    self.interpreter.load_program(&self.program).unwrap();
                    self.interpreter.set_output(Cursor::new(Vec::new()));
                }

                if ui.button("Step").clicked() {
                    self.interpreter.step().unwrap();
                }

                if ui.button("Run").clicked() {
                    self.running = true;
                }

                if ui.button("Stop").clicked() {
                    self.running = false;
                }

                if self.running {
                    ui.label("Running");
                } else {
                    ui.label("Not running");
                }
            });
        });

        egui::TopBottomPanel::bottom("bottom_panel")
            .min_height(100.0)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("Output");
                });
                ui.label(String::from_utf8_lossy(
                    self.interpreter.get_output().get_ref(),
                ));
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Program");
            });
            ui.add_sized(
                ui.available_size(),
                egui::TextEdit::multiline(&mut self.program).font(egui::TextStyle::Monospace),
            );
        });

        if self.running {
            self.interpreter.step().unwrap();

            if !self.interpreter.get_enabled() {
                self.running = false;
            }

            ctx.request_repaint();
        }
    }
}
