mod export;
mod logo;

use eframe::egui;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};

const SAVE_FILE: &str = "watches_save.json";
const LANG_FILE: &str = "language.txt";

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Lang {
    Fr,
    En,
    Es,
}

impl Lang {
    pub fn code(self) -> &'static str {
        match self {
            Lang::Fr => "fr",
            Lang::En => "en",
            Lang::Es => "es",
        }
    }

    fn load() -> Lang {
        match fs::read_to_string(LANG_FILE) {
            Ok(s) if s.trim() == "en" => Lang::En,
            Ok(s) if s.trim() == "es" => Lang::Es,
            _ => Lang::Fr,
        }
    }

    fn save(self) {
        let _ = fs::write(LANG_FILE, self.code());
    }
}

pub fn t(
    lang: Lang,
    fr: &'static str,
    en: &'static str,
    es: &'static str,
) -> &'static str {
    match lang {
        Lang::Fr => fr,
        Lang::En => en,
        Lang::Es => es,
    }
}

const DRIVES: [&str; 4] = ["Quartz", "Quartz solaire", "Automatique", "Mécanique"];

pub fn drive_label(lang: Lang, stored: &str) -> String {
    match (lang, stored) {
        (Lang::En, "Quartz solaire") => "Solar quartz".into(),
        (Lang::En, "Automatique") => "Automatic".into(),
        (Lang::En, "Mécanique") => "Mechanical".into(),
        (Lang::Es, "Quartz solaire") => "Cuarzo solar".into(),
        (Lang::Es, "Automatique") => "Automático".into(),
        (Lang::Es, "Mécanique") => "Mecánico".into(),
        _ => stored.to_string(),
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Watch {
    pub brand: String,
    pub model: String,
    pub dial_color: String,
    pub case_material: String,
    pub movement: String,
    pub drive_type: String,
    pub case_diameter: String,
    pub image_path: Option<String>,
    pub brand_image_path: Option<String>,
}

#[derive(PartialEq)]
enum ActiveModal {
    None,
    Add,
    Edit(usize),
}

#[derive(PartialEq)]
enum ViewMode {
    Cards,
    Table,
}

enum BgMsg {
    Logo(Result<PathBuf, String>),
}

pub struct WatchApp {
    lang: Lang,
    watches: Vec<Watch>,
    selected_index: Option<usize>,
    search_query: String,
    active_modal: ActiveModal,
    view_mode: ViewMode,

    form_brand: String,
    form_model: String,
    form_dial_color: String,
    form_case_material: String,
    form_movement: String,
    form_drive_type: String,
    form_diameter: String,
    form_image_path: Option<String>,
    form_brand_image_path: Option<String>,

    bg_tx: Sender<BgMsg>,
    bg_rx: Receiver<BgMsg>,
    busy_logo: bool,
    status: String,

    html_dialog: bool,
    textures: HashMap<String, egui::TextureHandle>,
}

const CARD_WIDTH: f32 = 250.0;
const PHOTO_BOX: f32 = 200.0;

impl WatchApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }

    fn save_to_file(&self) {
        if let Ok(json) = serde_json::to_string_pretty(&self.watches) {
            let _ = fs::write(SAVE_FILE, json);
        }
    }

    fn load_from_file() -> Vec<Watch> {
        if Path::new(SAVE_FILE).exists() {
            if let Ok(content) = fs::read_to_string(SAVE_FILE) {
                match serde_json::from_str::<Vec<Watch>>(&content) {
                    Ok(watches) => return watches,
                    Err(_) => {
                        let _ = fs::copy(SAVE_FILE, format!("{SAVE_FILE}.bak"));
                    }
                }
            }
        }
        vec![
            Watch {
                brand: "SteelDives".into(),
                model: "SD1970 (Willard)".into(),
                dial_color: "Noir".into(),
                case_material: "Acier 316L".into(),
                movement: "NH35A".into(),
                drive_type: "Automatique".into(),
                case_diameter: "44mm".into(),
                image_path: None,
                brand_image_path: None,
            },
            Watch {
                brand: "AddiesDive".into(),
                model: "AD2117".into(),
                dial_color: "Blanc Sable".into(),
                case_material: "Acier 316L".into(),
                movement: "NH35A".into(),
                drive_type: "Automatique".into(),
                case_diameter: "44mm".into(),
                image_path: None,
                brand_image_path: None,
            },
        ]
    }

    /// Thème moderne gris semi-transparent (Glassmorphism) avec angles arrondis prononcés
    fn apply_custom_theme(ctx: &egui::Context) {
        let mut visuals = egui::Visuals::dark();
        visuals.override_text_color = Some(egui::Color32::from_rgb(240, 240, 245));
        
        // Panneau et fond gris semi-transparents au lieu de noir
        visuals.panel_fill = egui::Color32::from_rgba_unmultiplied(65, 65, 75, 210);
        visuals.window_fill = egui::Color32::from_rgba_unmultiplied(80, 80, 95, 235);
        
        visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgba_unmultiplied(95, 95, 110, 180);
        visuals.widgets.inactive.bg_fill = egui::Color32::from_rgba_unmultiplied(110, 110, 125, 200);
        visuals.widgets.hovered.bg_fill = egui::Color32::from_rgba_unmultiplied(130, 130, 150, 220);
        visuals.widgets.active.bg_fill = egui::Color32::from_rgba_unmultiplied(212, 175, 55, 250); // Doré élégant

        // Arrondis généreux et modernes
        let corner_rounding = egui::Rounding::same(14.0);
        visuals.widgets.noninteractive.rounding = corner_rounding;
        visuals.widgets.inactive.rounding = corner_rounding;
        visuals.widgets.hovered.rounding = corner_rounding;
        visuals.widgets.active.rounding = corner_rounding;
        visuals.window_rounding = egui::Rounding::same(18.0);
        visuals.menu_rounding = egui::Rounding::same(12.0);

        ctx.set_visuals(visuals);
    }

    fn open_add_modal(&mut self) {
        let lang = self.lang;
        self.form_brand.clear();
        self.form_model.clear();
        self.form_dial_color = t(lang, "Noir", "Black", "Negro").into();
        self.form_case_material = t(lang, "Acier 316L", "316L steel", "Acero 316L").into();
        self.form_movement = "NH35A".into();
        self.form_drive_type = "Automatique".into();
        self.form_diameter = "40mm".into();
        self.form_image_path = None;
        self.form_brand_image_path = None;
        self.active_modal = ActiveModal::Add;
    }

    fn open_edit_modal(&mut self, idx: usize) {
        if let Some(w) = self.watches.get(idx) {
            self.form_brand = w.brand.clone();
            self.form_model = w.model.clone();
            self.form_dial_color = w.dial_color.clone();
            self.form_case_material = w.case_material.clone();
            self.form_movement = w.movement.clone();
            self.form_drive_type = w.drive_type.clone();
            self.form_diameter = w.case_diameter.clone();
            self.form_image_path = w.image_path.clone();
            self.form_brand_image_path = w.brand_image_path.clone();
            self.active_modal = ActiveModal::Edit(idx);
        }
    }

    fn load_texture_from_path(&mut self, ctx: &egui::Context, path: &str) -> Option<egui::TextureHandle> {
        if let Some(texture) = self.textures.get(path) {
            return Some(texture.clone());
        }
        if let Ok(image) = image::open(path) {
            let size = [image.width() as usize, image.height() as usize];
            let image_buffer = image.to_rgba8();
            let pixels = image_buffer.as_flat_samples();
            let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
            let texture = ctx.load_texture(path, color_image, egui::TextureOptions::default());
            self.textures.insert(path.to_string(), texture.clone());
            return Some(texture);
        }
        None
    }

    fn spawn_logo(&mut self, ctx: egui::Context) {
        let lang = self.lang;
        let brand = self.form_brand.trim().to_string();
        if brand.is_empty() {
            self.status = t(
                lang,
                "Saisis d'abord une marque",
                "Enter a brand first",
                "Introduce una marca primero"
            ).into();
            return;
        }
        self.busy_logo = true;
        self.status = match lang {
            Lang::Fr => format!("Recherche du logo de {brand}…"),
            Lang::En => format!("Looking for the {brand} logo…"),
            Lang::Es => format!("Buscando el logotipo de {brand}…"),
        };
        let tx = self.bg_tx.clone();
        std::thread::spawn(move || {
            let r = logo::fetch_logo(&brand).ok_or_else(|| match lang {
                Lang::Fr => format!("Logo introuvable pour {brand}"),
                Lang::En => format!("No logo found for {brand}"),
                Lang::Es => format!("Logotipo no encontrado para {brand}"),
            });
            let _ = tx.send(BgMsg::Logo(r));
            ctx.request_repaint();
        });
    }

    fn poll_background(&mut self) {
        let lang = self.lang;
        while let Ok(msg) = self.bg_rx.try_recv() {
            match msg {
                BgMsg::Logo(r) => {
                    self.busy_logo = false;
                    match r {
                        Ok(p) => {
                            self.form_brand_image_path = Some(p.display().to_string());
                            self.status = t(
                                lang,
                                "Logo trouvé ✔",
                                "Logo found ✔",
                                "Logotipo encontrado ✔"
                            ).into();
                        }
                        Err(e) => self.status = e,
                    }
                }
            }
        }
    }

    fn export_collection(&self) {
        if let Some(json_path) = rfd::FileDialog::new()
            .set_file_name("temps_libre_collection.json")
            .add_filter("JSON", &["json"])
            .save_file()
        {
            let base_dir = json_path.parent().unwrap_or(Path::new("."));
            let images_dir = base_dir.join("watch_images");
            let _ = fs::create_dir_all(&images_dir);

            let mut exported_watches = self.watches.clone();

            for (i, watch) in exported_watches.iter_mut().enumerate() {
                if let Some(path_str) = &watch.image_path {
                    let src = Path::new(path_str);
                    if src.exists() {
                        let extension = src.extension().and_then(|e| e.to_str()).unwrap_or("jpg");
                        let dest_filename = format!("watch_{}_{}.{}", i, uuid_simple(), extension);
                        let dest = images_dir.join(&dest_filename);
                        if fs::copy(src, &dest).is_ok() {
                            watch.image_path = Some(format!("watch_images/{}", dest_filename));
                        }
                    }
                }
                if let Some(path_str) = &watch.brand_image_path {
                    let src = Path::new(path_str);
                    if src.exists() {
                        let extension = src.extension().and_then(|e| e.to_str()).unwrap_or("png");
                        let dest_filename = format!("brand_{}_{}.{}", i, uuid_simple(), extension);
                        let dest = images_dir.join(&dest_filename);
                        if fs::copy(src, &dest).is_ok() {
                            watch.brand_image_path = Some(format!("watch_images/{}", dest_filename));
                        }
                    }
                }
            }

            if let Ok(json) = serde_json::to_string_pretty(&exported_watches) {
                let _ = fs::write(json_path, json);
            }
        }
    }

    fn import_collection(&mut self) {
        if let Some(json_path) = rfd::FileDialog::new()
            .add_filter("JSON", &["json"])
            .pick_file()
        {
            if let Ok(content) = fs::read_to_string(&json_path) {
                if let Ok(mut imported_watches) = serde_json::from_str::<Vec<Watch>>(&content) {
                    let base_dir = json_path.parent().unwrap_or(Path::new("."));

                    for watch in imported_watches.iter_mut() {
                        if let Some(path_str) = &watch.image_path {
                            let p = Path::new(path_str);
                            if !p.is_absolute() {
                                let absolute_p = base_dir.join(p);
                                if absolute_p.exists() {
                                    watch.image_path = Some(absolute_p.display().to_string());
                                }
                            }
                        }
                        if let Some(path_str) = &watch.brand_image_path {
                            let p = Path::new(path_str);
                            if !p.is_absolute() {
                                let absolute_p = base_dir.join(p);
                                if absolute_p.exists() {
                                    watch.brand_image_path = Some(absolute_p.display().to_string());
                                }
                            }
                        }
                    }

                    self.watches = imported_watches;
                    self.selected_index = None;
                    self.save_to_file();
                }
            }
        }
    }
}

impl Default for WatchApp {
    fn default() -> Self {
        let (bg_tx, bg_rx) = channel();
        Self {
            lang: Lang::load(),
            watches: Self::load_from_file(),
            selected_index: None,
            search_query: "".into(),
            active_modal: ActiveModal::None,
            view_mode: ViewMode::Cards,

            form_brand: "".into(),
            form_model: "".into(),
            form_dial_color: "Noir".into(),
            form_case_material: "Acier 316L".into(),
            form_movement: "NH35A".into(),
            form_drive_type: "Automatique".into(),
            form_diameter: "40mm".into(),
            form_image_path: None,
            form_brand_image_path: None,

            bg_tx,
            bg_rx,
            busy_logo: false,
            status: String::new(),

            html_dialog: false,
            textures: HashMap::new(),
        }
    }
}

fn uuid_simple() -> u32 {
    rand::random::<u32>()
}

impl eframe::App for WatchApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        Self::apply_custom_theme(ctx);
        self.poll_background();

        let lang = self.lang;
        let mut index_to_delete = None;
        let mut new_lang: Option<Lang> = None;

        egui::TopBottomPanel::top("top_header").show(ctx, |ui| {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.heading("⏳ TEMPS LIBRE");
                ui.label(format!(
                    "|  {} {}",
                    self.watches.len(),
                    t(lang, "pièces", "watches", "piezas")
                ));

                ui.add_space(15.0);

                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        if ui
                            .selectable_label(self.view_mode == ViewMode::Cards, t(lang, "🗂️ Cartes", "🗂️ Cards", "🗂️ Tarjetas"))
                            .clicked()
                        {
                            self.view_mode = ViewMode::Cards;
                        }
                        if ui
                            .selectable_label(self.view_mode == ViewMode::Table, t(lang, "📋 Tableau", "📋 Table", "📋 Tabla"))
                            .clicked()
                        {
                            self.view_mode = ViewMode::Table;
                        }
                    });
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.selectable_label(lang == Lang::Es, "ES").clicked() {
                        new_lang = Some(Lang::Es);
                    }
                    if ui.selectable_label(lang == Lang::En, "EN").clicked() {
                        new_lang = Some(Lang::En);
                    }
                    if ui.selectable_label(lang == Lang::Fr, "FR").clicked() {
                        new_lang = Some(Lang::Fr);
                    }
                    ui.add_space(10.0);

                    if ui.button(t(lang, "➕ Ajouter", "➕ Add", "➕ Añadir")).clicked() {
                        self.open_add_modal();
                    }
                    if ui.button(t(lang, "📥 Importer", "📥 Import", "📥 Importar")).clicked() {
                        self.import_collection();
                    }
                    if ui.button("📤 JSON").clicked() {
                        self.export_collection();
                    }
                    if ui.button("🌐 HTML").clicked() {
                        self.html_dialog = true;
                    }
                    ui.add_space(10.0);
                    ui.text_edit_singleline(&mut self.search_query);
                    ui.label("🔍");
                });
            });
            ui.add_space(8.0);
        });

        egui::TopBottomPanel::bottom("bottom_bar").show(ctx, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                if !self.status.is_empty() {
                    ui.label(&self.status);
                }
            });
            ui.add_space(4.0);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add_space(10.0);

                let query = self.search_query.to_lowercase();
                let filtered_indices: Vec<usize> = self
                    .watches
                    .iter()
                    .enumerate()
                    .filter(|(_, w)| {
                        query.is_empty()
                            || w.brand.to_lowercase().contains(&query)
                            || w.model.to_lowercase().contains(&query)
                            || w.movement.to_lowercase().contains(&query)
                            || w.dial_color.to_lowercase().contains(&query)
                            || w.drive_type.to_lowercase().contains(&query)
                            || drive_label(lang, &w.drive_type).to_lowercase().contains(&query)
                    })
                    .map(|(i, _)| i)
                    .collect();

                if filtered_indices.is_empty() {
                    ui.centered_and_justified(|ui| {
                        ui.label(t(
                            lang,
                            "Aucune montre ne correspond à votre recherche.",
                            "No watch matches your search.",
                            "Ningún reloj coincide con tu búsqueda."
                        ));
                    });
                } else {
                    match self.view_mode {
                        ViewMode::Cards => {
                            let cards: Vec<(usize, Watch)> = filtered_indices
                                .iter()
                                .map(|&i| (i, self.watches[i].clone()))
                                .collect();

                            ui.horizontal_wrapped(|ui| {
                                for (idx, w) in cards {
                                    egui::Frame::none()
                                        .fill(egui::Color32::from_rgba_unmultiplied(85, 85, 100, 210)) // Fond de carte gris
                                        .rounding(egui::Rounding::same(16.0)) // Coins très arrondis
                                        .inner_margin(14.0)
                                        .stroke(egui::Stroke::new(
                                            1.0_f32,
                                            if self.selected_index == Some(idx) {
                                                egui::Color32::from_rgb(212, 175, 55)
                                            } else {
                                                egui::Color32::from_rgba_unmultiplied(120, 120, 140, 150)
                                            },
                                        ))
                                        .show(ui, |ui| {
                                            ui.set_width(CARD_WIDTH);

                                            ui.vertical_centered(|ui| {
                                                let mut photo_drawn = false;
                                                if let Some(path) = &w.image_path {
                                                    if let Some(texture) = self.load_texture_from_path(ctx, path) {
                                                        let size = texture.size_vec2();
                                                        let k = (PHOTO_BOX / size.x).min(PHOTO_BOX / size.y);
                                                        ui.image((texture.id(), size * k));
                                                        photo_drawn = true;
                                                    }
                                                }
                                                if !photo_drawn {
                                                    ui.allocate_space(egui::vec2(PHOTO_BOX, PHOTO_BOX));
                                                }

                                                ui.add_space(6.0);
                                                ui.horizontal(|ui| {
                                                    if let Some(p) = &w.brand_image_path {
                                                        if let Some(tex) = self.load_texture_from_path(ctx, p) {
                                                            ui.image((tex.id(), egui::vec2(20.0, 20.0)));
                                                        }
                                                    }
                                                    ui.strong(&w.brand);
                                                });
                                                ui.label(&w.model);
                                            });

                                            ui.add_space(10.0);
                                            ui.horizontal(|ui| {
                                                if ui.small_button(t(lang, "✏️ Modifier", "✏️ Edit", "✏️ Editar")).clicked() {
                                                    self.open_edit_modal(idx);
                                                }
                                                if ui.small_button(t(lang, "🗑 Suppr", "🗑 Delete", "🗑 Borrar")).clicked() {
                                                    index_to_delete = Some(idx);
                                                }
                                            });
                                        });

                                    ui.add_space(10.0);
                                }
                            });
                        }
                        ViewMode::Table => {
                            egui::Grid::new("watches_table_view")
                                .striped(true)
                                .spacing([15.0, 10.0])
                                .show(ui, |ui| {
                                    for h in [
                                        t(lang, "Marque", "Brand", "Marca"),
                                        t(lang, "Modèle", "Model", "Modelo"),
                                        t(lang, "Cadran", "Dial", "Esfera"),
                                        t(lang, "Boîtier", "Case", "Caja"),
                                        t(lang, "Mouvement", "Movement", "Movimiento"),
                                        t(lang, "Entraînement", "Drive", "Mecanismo"),
                                        t(lang, "Diamètre", "Diameter", "Diámetro"),
                                        t(lang, "Actions", "Actions", "Acciones"),
                                    ] {
                                        ui.strong(h);
                                    }
                                    ui.end_row();

                                    for &idx in &filtered_indices {
                                        let watch = &self.watches[idx];

                                        ui.label(&watch.brand);
                                        ui.label(&watch.model);
                                        ui.label(&watch.dial_color);
                                        ui.label(&watch.case_material);
                                        ui.label(&watch.movement);
                                        ui.label(drive_label(lang, &watch.drive_type));
                                        ui.label(&watch.case_diameter);

                                        ui.horizontal(|ui| {
                                            if ui.small_button("✏️").clicked() {
                                                self.open_edit_modal(idx);
                                            }
                                            if ui.small_button("🗑").clicked() {
                                                index_to_delete = Some(idx);
                                            }
                                        });
                                        ui.end_row();
                                    }
                                });
                        }
                    }
                }
            });
        });

        if let Some(idx) = index_to_delete {
            self.watches.remove(idx);
            self.save_to_file();
        }

        if let Some(l) = new_lang {
            if l != self.lang {
                self.lang = l;
                l.save();
            }
        }

        if self.html_dialog {
            let mut open = true;
            let mut go = false;
            egui::Window::new(t(
                lang,
                "🌐 Exporter en page HTML",
                "🌐 Export as HTML page",
                "🌐 Exportar como página HTML"
            ))
                .id(egui::Id::new("html_export_window"))
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .open(&mut open)
                .show(ctx, |ui| {
                    ui.label(t(
                        lang,
                        "Crée une page unique (photos incluses), à ouvrir dans un navigateur ou à héberger.",
                        "Creates a single page (photos included) to open in a browser or host online.",
                        "Crea una página única (fotos incluidas) para abrir en un navegador o alojar online."
                    ));
                    ui.add_space(10.0);
                    if ui.button(t(lang, "📄 Exporter…", "📄 Export…", "📄 Exportar…")).clicked() {
                        go = true;
                    }
                });
            if go {
                self.status = match export::export_html(&self.watches, lang) {
                    Ok(Some(path)) => {
                        let _ = open::that(&path);
                        match lang {
                            Lang::Fr => format!("Page exportée : {}", path.display()),
                            Lang::En => format!("Page exported: {}", path.display()),
                            Lang::Es => format!("Página exportada: {}", path.display()),
                        }
                    }
                    Ok(None) => t(
                        lang,
                        "Export annulé",
                        "Export cancelled",
                        "Exportación cancelada"
                    ).into(),
                    Err(e) => e,
                };
                self.html_dialog = false;
            }
            if !open {
                self.html_dialog = false;
            }
        }

        if self.active_modal != ActiveModal::None {
            let is_edit = matches!(self.active_modal, ActiveModal::Edit(_));
            let title = if is_edit {
                t(
                    lang,
                    "✏️ Modifier la montre",
                    "✏️ Edit watch",
                    "✏️ Editar reloj"
                )
            } else {
                t(
                    lang,
                    "➕ Nouvelle montre",
                    "➕ New watch",
                    "➕ Nuevo reloj"
                )
            };

            let mut close_modal = false;
            let mut save_modal = false;

            egui::Window::new(title)
                .id(egui::Id::new("watch_form_window"))
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(ctx, |ui| {
                    egui::Grid::new("form_grid").num_columns(2).spacing([12.0, 8.0]).show(ui, |ui| {
                        ui.label(t(lang, "Marque :", "Brand:", "Marca:"));
                        ui.text_edit_singleline(&mut self.form_brand);
                        ui.end_row();

                        ui.label(t(lang, "Modèle :", "Model:", "Modelo:"));
                        ui.text_edit_singleline(&mut self.form_model);
                        ui.end_row();

                        ui.label(t(lang, "Cadran :", "Dial:", "Esfera:"));
                        ui.text_edit_singleline(&mut self.form_dial_color);
                        ui.end_row();

                        ui.label(t(lang, "Boîtier :", "Case:", "Caja:"));
                        ui.text_edit_singleline(&mut self.form_case_material);
                        ui.end_row();

                        ui.label(t(lang, "Mouvement :", "Movement:", "Movimiento:"));
                        ui.text_edit_singleline(&mut self.form_movement);
                        ui.end_row();

                        ui.label(t(lang, "Entraînement :", "Drive:", "Mecanismo:"));
                        egui::ComboBox::from_id_source("modal_drive_combo")
                            .selected_text(drive_label(lang, &self.form_drive_type))
                            .show_ui(ui, |ui| {
                                for d in DRIVES {
                                    ui.selectable_value(
                                        &mut self.form_drive_type,
                                        d.to_string(),
                                        drive_label(lang, d),
                                    );
                                }
                            });
                        ui.end_row();

                        ui.label(t(lang, "Diamètre :", "Diameter:", "Diámetro:"));
                        ui.text_edit_singleline(&mut self.form_diameter);
                        ui.end_row();

                        ui.label(t(lang, "Photo :", "Photo:", "Foto:"));
                        ui.horizontal(|ui| {
                            if ui.button(t(lang, "📁 Parcourir...", "📁 Browse...", "📁 Explorar...")).clicked() {
                                if let Some(path) = rfd::FileDialog::new()
                                    .add_filter("Images", &["png", "jpg", "jpeg", "webp"])
                                    .pick_file()
                                {
                                    self.form_image_path = Some(path.display().to_string());
                                }
                            }
                            if self.form_image_path.is_some() {
                                ui.label("✔");
                            }
                        });
                        ui.end_row();

                        ui.label(t(lang, "Logo marque :", "Brand logo:", "Logo de marca:"));
                        ui.horizontal(|ui| {
                            if ui
                                .button("📁")
                                .on_hover_text(t(lang, "Choisir un fichier", "Choose a file", "Elegir un archivo"))
                                .clicked()
                            {
                                if let Some(path) = rfd::FileDialog::new()
                                    .add_filter("Images", &["png", "jpg", "jpeg", "webp"])
                                    .pick_file()
                                {
                                    self.form_brand_image_path = Some(path.display().to_string());
                                }
                            }
                            let label = if self.busy_logo { "⏳" } else { "🔎 Auto" };
                            if ui
                                .add_enabled(!self.busy_logo, egui::Button::new(label))
                                .on_hover_text(t(
                                    lang,
                                    "Cherche le logo sur le site de la marque",
                                    "Looks for the logo on the brand's website",
                                    "Busca el logotipo en el sitio web de la marca"
                                ))
                                .clicked()
                            {
                                self.spawn_logo(ctx.clone());
                            }
                            if self.form_brand_image_path.is_some() {
                                ui.label("✔");
                            }
                        });
                        ui.end_row();
                    });

                    ui.add_space(15.0);
                    ui.horizontal(|ui| {
                        if ui.button(t(lang, "💾 Enregistrer", "💾 Save", "💾 Guardar")).clicked() {
                            save_modal = true;
                        }
                        if ui.button(t(lang, "Annuler", "Cancel", "Cancelar")).clicked() {
                            close_modal = true;
                        }
                    });
                });

            if save_modal {
                let updated_watch = Watch {
                    brand: self.form_brand.clone(),
                    model: self.form_model.clone(),
                    dial_color: self.form_dial_color.clone(),
                    case_material: self.form_case_material.clone(),
                    movement: self.form_movement.clone(),
                    drive_type: self.form_drive_type.clone(),
                    case_diameter: self.form_diameter.clone(),
                    image_path: self.form_image_path.clone(),
                    brand_image_path: self.form_brand_image_path.clone(),
                };

                match self.active_modal {
                    ActiveModal::Add => {
                        self.watches.push(updated_watch);
                    }
                    ActiveModal::Edit(idx) => {
                        if let Some(w) = self.watches.get_mut(idx) {
                            *w = updated_watch;
                        }
                    }
                    ActiveModal::None => {}
                }

                self.save_to_file();
                self.active_modal = ActiveModal::None;
            } else if close_modal {
                self.active_modal = ActiveModal::None;
            }
        }
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 750.0])
            .with_transparent(true),
        ..Default::default()
    };
    eframe::run_native(
        "Temps Libre - Gestionnaire de Collection",
        options,
        Box::new(|cc| Ok(Box::new(WatchApp::new(cc)))),
    )
}