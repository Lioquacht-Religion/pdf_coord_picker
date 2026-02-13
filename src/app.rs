use std::path::PathBuf;
use std::sync::mpsc;

use egui::{
    Color32, Painter, PointerButton, Pos2, Rect, Response, Sense, Stroke, TextEdit, TextureHandle,
    epaint,
};
use image::DynamicImage;
use pdfium_render::prelude::PdfDocument;
use pdfium_render::prelude::{PdfRenderConfig, PdfiumError};
use slotmap::{DenseSlotMap, new_key_type};

#[cfg(not(target_arch = "wasm32"))]
use crate::file_dialog;

#[cfg(target_arch = "wasm32")]
use crate::file_dialog;

pub enum PdfLoadError {
    FileError,
    PdfError((PathBuf, PdfiumError)),
}

pub type PdfFileLoadType = Result<(PathBuf, Vec<DynamicImage>), PdfLoadError>;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct PdfCoordPickerApp {
    page_max_width: String,
    page_max_height: String,
    pub pdf_file_path: Option<PathBuf>,
    #[serde(skip)]
    pub pdf_page_textures: Option<Vec<PdfPageImage>>,
    #[serde(skip)]
    selected_page_input_id: Option<PdfPageInputId>,

    pub waiting_for_file: bool,
    #[serde(skip)]
    pub receiver: mpsc::Receiver<PdfFileLoadType>,
    #[serde(skip)]
    pub producer: mpsc::Sender<PdfFileLoadType>,
}

impl Default for PdfCoordPickerApp {
    fn default() -> Self {
        let (mp, sc) = mpsc::channel();
        Self {
            page_max_width: String::new(),
            page_max_height: String::new(),
            pdf_file_path: None,
            pdf_page_textures: None,
            selected_page_input_id: None,
            waiting_for_file: false,
            receiver: sc,
            producer: mp,
        }
    }
}

impl PdfCoordPickerApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        }
    }

    pub fn init_pdf_page_images(
        &mut self,
        ctx: &egui::Context,
        pdf_page_images: Vec<DynamicImage>,
    ) {
        if let Ok(pdf_page_image) = load_pdf_page_image(ctx, pdf_page_images) {
            self.pdf_page_textures = Some(pdf_page_image);
        } else {
            self.pdf_page_textures = None;
        }
    }

    fn get_input_field_mut(&mut self, key: PdfPageInputId) -> Option<&mut PdfInputField> {
        if let Some(pages) = &mut self.pdf_page_textures {
            if let Some(page) = pages.get_mut(key.page_id) {
                page.input_fields.get_mut(key.input_field_key)
            } else {
                None
            }
        } else {
            None
        }
    }
}

pub struct PdfPageImage {
    texture_handle: TextureHandle,
    width: f32,
    height: f32,
    input_fields: DenseSlotMap<PdfInputFieldKey, PdfInputField>,
}

impl PdfPageImage {
    fn new(texture_handle: TextureHandle, width: f32, height: f32) -> Self {
        PdfPageImage {
            texture_handle,
            width,
            height,
            input_fields: DenseSlotMap::default(),
        }
    }
}

new_key_type! { struct PdfInputFieldKey; }

#[derive(Debug, Clone, Copy)]
struct PdfPageInputId {
    page_id: usize,
    input_field_key: PdfInputFieldKey,
}

struct PdfInputField {
    rect: Rect,
    text: String,
}

pub fn create_images_from_pdf<'a>(
    pdf_document: PdfDocument<'a>,
) -> Result<Vec<DynamicImage>, PdfiumError> {
    let mut images = Vec::with_capacity(pdf_document.pages().len() as usize);
    let mut pages = pdf_document.pages().iter();
    while let Some(page) = pages.next() {
        match page.render_with_config(&PdfRenderConfig::new()) {
            Ok(pdfbitmap) => images.push(pdfbitmap.as_image()),
            Err(e) => return Err(e),
        }
    }
    Ok(images)
}

fn load_pdf_page_image(
    ctx: &egui::Context,
    pdf_page_images: Vec<DynamicImage>,
) -> Result<Vec<PdfPageImage>, PdfiumError> {
    Ok(pdf_page_images
        .into_iter()
        .map(|image| {
            let color_image = convert_to_color_image(image);
            let width = color_image.width() as f32;
            let height = color_image.height() as f32;

            let handle = ctx.load_texture("KESTPDF", color_image, egui::TextureOptions::default());
            PdfPageImage::new(handle, width, height)
        })
        .collect())
}

fn convert_to_color_image(image: DynamicImage) -> egui::ColorImage {
    use image::EncodableLayout;
    let color_image = match &image {
        DynamicImage::ImageRgb8(image) => {
            // common case optimization
            egui::ColorImage::from_rgb(
                [image.width() as usize, image.height() as usize],
                image.as_bytes(),
            )
        }
        other => {
            let image = other.to_rgba8();
            egui::ColorImage::from_rgba_unmultiplied(
                [image.width() as usize, image.height() as usize],
                image.as_bytes(),
            )
        }
    };
    color_image
}

impl eframe::App for PdfCoordPickerApp {
    /// Called by the framework to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                #[cfg(not(target_arch = "wasm32"))]
                file_dialog::file_dialog_native::handle_open_file_dialog_native(self, ctx, ui);
                #[cfg(target_arch = "wasm32")]
                file_dialog::file_dialog_web::handle_open_file_dialog_web(self, ctx, ui);
                ui.add_space(16.0);

                egui::widgets::global_theme_preference_buttons(ui);
            });
        });

        egui::SidePanel::right("right_side_panel")
            .resizable(true)
            .show(ctx, |ui| {
                if let Some(key) = self.selected_page_input_id {
                    if let Some(input_field) = self.get_input_field_mut(key) {
                        ui.label(format!(
                            "page id: {}; input id: {:?}",
                            key.page_id.clone(),
                            key.input_field_key
                        ));
                        ui.text_edit_singleline(&mut input_field.text);
                    } else {
                        ui.label(format!("page id: {};", key.page_id.clone()));
                        ui.label("Selected input field does not exist anymore.");
                    }
                } else {
                    ui.label("No input field is selected.");
                }
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading("PDF Coordinates Picker");

            ui.separator();

            ui.horizontal(|ui| {
                // custom max width
                ui.label("max width: ");
                ui.text_edit_singleline(&mut self.page_max_width);
                // custom max height
                ui.label("max height: ");
                ui.text_edit_singleline(&mut self.page_max_height);
            });

            ui.separator();

            draw_pdf_pages(self, ui);

            ui.separator();

            ui.add(egui::github_link_file!(
                "https://github.com/emilk/eframe_template/blob/main/",
                "Source code."
            ));

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                powered_by_egui_and_eframe(ui);
                egui::warn_if_debug_build(ui);
            });
        });
    }
}

fn draw_pdf_pages(app: &mut PdfCoordPickerApp, ui: &mut egui::Ui) {
    //"KapSt_2021_Entwurf.pdf"
    let file_path = if let Some(file_path) = &app.pdf_file_path {
        file_path
    } else {
        return;
    };
    let pdf_page_images: &mut Vec<PdfPageImage> =
        if let Some(pdf_page_image) = &mut app.pdf_page_textures {
            pdf_page_image
        } else {
            ui.label(format!(
                "File {} could not be loaded.",
                file_path.to_string_lossy()
            ));
            return;
        };

    let row_height = pdf_page_images.first().map(|img| img.height).unwrap_or(0.);
    let row_num = pdf_page_images.len();
    egui::ScrollArea::vertical().auto_shrink(true).show_rows(
        ui,
        row_height,
        row_num,
        |ui, row_range| {
            for row in row_range {
                let page = &mut pdf_page_images[row];
                let sized_image = egui::load::SizedTexture::new(
                    page.texture_handle.id(),
                    egui::vec2(page.width, page.height),
                );
                let image = egui::Image::from_texture(sized_image);

                let (response, mut painter) = ui.allocate_painter(
                    egui::Vec2::new(page.width, page.height),
                    Sense::click() | Sense::hover(),
                );
                image.paint_at(ui, response.rect);

                draw_pdf_input_fields(
                    &response,
                    &mut painter,
                    row,
                    &mut page.input_fields,
                    &mut app.selected_page_input_id,
                    ui,
                );

                let _response = handle_pdf_input_create(
                    response,
                    page,
                    &app.page_max_width,
                    &app.page_max_height,
                );
            }
        },
    );
}

fn handle_pdf_input_create(
    mut pdf_page_response: Response,
    pdf_page: &mut PdfPageImage,
    max_width: &str,
    max_height: &str,
) -> Response {
    if let Some(Pos2 { x, y }) = pdf_page_response.interact_pointer_pos() {
        println!("clicked mouse in rectangle got pos");
        let delta_x = (pdf_page_response.rect.left() - x).abs();
        let delta_y = (pdf_page_response.rect.top() - y).abs();

        if pdf_page_response.clicked_by(PointerButton::Primary) {
            let delta_pos = (delta_x, delta_y).into();
            println!("clicked mouse in rectangle");
            pdf_page.input_fields.insert(PdfInputField {
                rect: Rect::from_center_size(delta_pos, [30., 30.].into()),
                text: String::new(),
            });
        }
    }

    if let Some(Pos2 { x, y }) = pdf_page_response.hover_pos() {
        let max_width: f32 = max_width.parse().unwrap_or(pdf_page.width);
        let max_height: f32 = max_height.parse().unwrap_or(pdf_page.height);

        let width_fraction: f32 = max_width / pdf_page.width;
        let height_fraction: f32 = max_height / pdf_page.height;

        let delta_x = (x - pdf_page_response.rect.left()).abs() * width_fraction;
        let delta_y = (y - pdf_page_response.rect.top()).abs() * height_fraction;
        pdf_page_response = pdf_page_response.on_hover_ui_at_pointer(|ui| {
            ui.label(format!("x: {delta_x}; y: {delta_y};"));
        });
    }

    pdf_page_response
}

fn draw_pdf_input_fields(
    response: &Response,
    painter: &mut Painter,
    page_id: usize,
    pdf_input_fields: &mut DenseSlotMap<PdfInputFieldKey, PdfInputField>,
    selected_input_field_key: &mut Option<PdfPageInputId>,
    ui: &mut egui::Ui,
) {
    for (key, input_field) in pdf_input_fields {
        let input_field_response = ui_draw_input_field(response, painter, input_field, ui);
        check_input_field_click(
            &input_field_response,
            selected_input_field_key,
            PdfPageInputId {
                page_id,
                input_field_key: key,
            },
        );
    }
}

fn ui_draw_input_field(
    response: &Response,
    painter: &mut Painter,
    pdf_input_field: &mut PdfInputField,
    ui: &mut egui::Ui,
) -> Response {
    let translate_field_rect = pdf_input_field
        .rect
        .translate((response.rect.left(), response.rect.top()).into());
    painter.add(epaint::RectShape::stroke(
        translate_field_rect,
        0.0,
        Stroke::new(3., Color32::from_rgb(255, 20, 20)),
        egui::StrokeKind::Outside,
    ));
    ui.place(
        translate_field_rect,
        TextEdit::singleline(&mut pdf_input_field.text),
    )
}

fn check_input_field_click(
    response: &Response,
    selected_input_field_key: &mut Option<PdfPageInputId>,
    input_field_key: PdfPageInputId,
) {
    if response.clicked() {
        *selected_input_field_key = Some(input_field_key);
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}
