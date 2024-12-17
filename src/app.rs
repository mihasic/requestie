use strum::{Display, EnumIter, EnumString, IntoEnumIterator};

#[derive(
    PartialEq, Debug, Clone, serde::Serialize, serde::Deserialize, Display, EnumString, EnumIter,
)]
pub enum HttpMethod {
    GET,
    HEAD,
    PUT,
    POST,
    PATCH,
    DELETE,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
struct Request {
    name: String,
    url: String,
    method: HttpMethod,
    headers: Vec<(String, String)>,
    body: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
struct Environment {
    name: String,
    values: Vec<(String, String)>,
}

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
enum PanelSelection {
    Request(usize),
    Environment(usize),
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
#[serde(default)]
pub struct RequestieApp {
    requests: Vec<Request>,
    environments: Vec<Environment>,
    selected_panel: PanelSelection,
}

impl Default for RequestieApp {
    fn default() -> Self {
        Self {
            requests: vec![Request {
                name: "Request 1".to_string(),
                url: "https://example.com".to_string(),
                method: HttpMethod::GET,
                headers: vec![("Content-Type".to_string(), "application/json".to_string())],
                body: "".to_string(),
            }],
            environments: vec![Environment {
                name: "Default".to_string(),
                values: vec![],
            }],
            selected_panel: PanelSelection::Request(0),
        }
    }
}

impl RequestieApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }

    fn show_request_editor(&mut self, ui: &mut eframe::egui::Ui, idx: usize) {
        let request = &mut self.requests[idx];

        egui::TopBottomPanel::top("header")
            .show_inside(ui, |ui| {
                ui.heading(&request.name);
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut request.name);
                });
        
            });

        ui.horizontal(|ui| {
            eframe::egui::ComboBox::from_label("")
                .selected_text(request.method.to_string())
                .show_ui(ui, |ui| {
                    for method in HttpMethod::iter() {
                        ui.selectable_value(
                            &mut request.method,
                            method.clone(),
                            method.to_string(),
                        );
                    }
                });

            ui.text_edit_singleline(&mut request.url);
        });

        ui.label("Headers:");
        let height = egui::TextStyle::Body.resolve(ui.style()).size;
        let mut to_remove = Vec::new();
        egui_extras::TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::LEFT))
            .column(egui_extras::Column::remainder())
            .column(egui_extras::Column::remainder())
            .column(egui_extras::Column::auto())
            .body(|mut body| {
                for (idx, header) in request.headers.iter_mut().enumerate() {
                    body.row(height, |mut row| {
                        row.col(|ui| {
                            ui.text_edit_singleline(&mut header.0);
                        });
                        row.col(|ui| {
                            ui.text_edit_singleline(&mut header.1);
                        });
                        row.col(|ui| {
                            if ui.button("❌").clicked() {
                                to_remove.push(idx);
                            }
                        });
                    });
                }
            });

        for idx in to_remove.into_iter().rev() {
            request.headers.remove(idx);
        }

        if ui.button("➕ Add Header").clicked() {
            request.headers.push(("".to_owned(), "".to_owned()));
        }

        ui.label("Body:");
        ui.text_edit_multiline(&mut request.body);
    }

    fn show_environment_editor(&mut self, ui: &mut eframe::egui::Ui, idx: usize) {
        let allow_delete = self.environments.len() > 1;
        let environment = &mut self.environments[idx];

        ui.heading(&environment.name);

        let mut delete_environment = false;
        ui.horizontal(|ui| {
            ui.text_edit_singleline(&mut environment.name);

            if allow_delete && ui.button("❌").clicked() {
                delete_environment = true;
            }
        });

        if delete_environment {
            self.environments.remove(idx);
            self.selected_panel = PanelSelection::Environment(0);
            return;
        }

        let mut to_remove = Vec::new();

        eframe::egui::Grid::new("env_grid")
            .min_col_width(ui.available_width() / 2.0)
            .show(ui, |ui| {
                for (idx, value) in environment.values.iter_mut().enumerate() {
                    ui.text_edit_singleline(&mut value.0);
                    ui.text_edit_singleline(&mut value.1);
                    if ui.button("❌").clicked() {
                        to_remove.push(idx);
                    }
                    ui.end_row();
                }
            });

        for idx in to_remove.into_iter().rev() {
            environment.values.remove(idx);
        }

        if ui.button("➕ Add Row").clicked() {
            environment.values.push(("".to_owned(), "".to_owned()));
        }
    }
}

impl eframe::App for RequestieApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                egui::widgets::global_theme_preference_buttons(ui);
            });
        });
        eframe::egui::SidePanel::left("left_panel").show(ctx, |ui| {
            ui.heading("Requests");
            let mut delete_idx: Option<usize> = None;

            for (idx, request) in self.requests.iter().enumerate() {
                if ui
                    .selectable_label(
                        self.selected_panel == PanelSelection::Request(idx),
                        &request.name,
                    )
                    .clicked()
                {
                    self.selected_panel = PanelSelection::Request(idx);
                }
            }

            if ui.button("+ Request").clicked() {
                self.requests.push(Request {
                    name: format!("New Request {}", self.requests.len() + 1),
                    url: "".to_string(),
                    method: HttpMethod::GET,
                    headers: vec![],
                    body: "".to_string(),
                });
                self.selected_panel = PanelSelection::Request(self.requests.len() - 1);
            }

            ui.separator();

            ui.heading("Environments");
            for (idx, env) in self.environments.iter().enumerate() {
                if ui
                    .selectable_label(
                        self.selected_panel == PanelSelection::Environment(idx),
                        &env.name,
                    )
                    .clicked()
                {
                    self.selected_panel = PanelSelection::Environment(idx);
                }
            }

            if ui.button("+ Environment").clicked() {
                self.environments.push(Environment {
                    name: format!("New Environment {}", self.environments.len() + 1),
                    values: vec![],
                });
                self.selected_panel = PanelSelection::Environment(self.environments.len() - 1);
            }
        });

        eframe::egui::CentralPanel::default().show(ctx, |ui| match self.selected_panel {
            PanelSelection::Request(idx) => self.show_request_editor(ui, idx),
            PanelSelection::Environment(idx) => self.show_environment_editor(ui, idx),
        });
    }
}
