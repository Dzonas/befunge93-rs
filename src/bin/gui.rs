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
    interpreter: Interpreter<Cursor<Vec<u8>>, Cursor<Vec<u8>>, ThreadRng>,
    running: bool,
}

impl Befunge93App {
    fn build_interpreter() -> Interpreter<Cursor<Vec<u8>>, Cursor<Vec<u8>>, ThreadRng> {
        let input = Cursor::new(Vec::new());
        // let input = io::stdin().lock();
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
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Program");
            ui.text_edit_multiline(&mut self.program);

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
            });
            ui.separator();
            ui.code(String::from_utf8_lossy(
                self.interpreter.get_output().get_ref(),
            ));
            ui.label(
                self.interpreter
                    .get_stack()
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>()
                    .join(" "),
            );

            if self.running {
                ui.label("Running");
            } else {
                ui.label("Not running");
            }
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
