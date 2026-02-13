// file_dialog_native.rs

use pdfium_render::prelude::Pdfium;

use crate::app::{PdfLoadError, create_images_from_pdf};
use crate::{PdfCoordPickerApp, pdf_load};
use std::sync::mpsc::TryRecvError;

pub fn handle_open_file_dialog_native(
    app: &mut PdfCoordPickerApp,
    ctx: &egui::Context,
    ui: &mut egui::Ui,
) {
    ui.menu_button("File", |ui| {
        if ui.button("Open file…").clicked()
        //TODO: execute file dialog in seperate thread and return picked file
        //TODO: wasm support
        {
            spawn_file_dialog_thread(app);

            // NOTE: no File->Quit on web pages!
            if ui.button("Quit").clicked() {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
    });
    if app.waiting_for_file{
        handle_file_load_error(app, ctx, ui);
    }
}

fn spawn_file_dialog_thread(app: &mut PdfCoordPickerApp) {
    app.waiting_for_file = true;
    let mp = app.producer.clone();
    std::thread::spawn(move || {

        let result = if let Some(path) = rfd::FileDialog::new().pick_file() {
            match pdf_load::load_pdf_native(&Pdfium::default(), &path) {
                Ok(pdf_document) => match create_images_from_pdf(pdf_document) {
                    Ok(pdf_images) => Ok((path, pdf_images)),
                    Err(e) => Err(PdfLoadError::PdfError((path, e))),
                },
                Err(e) => Err(PdfLoadError::PdfError((path, e))),
            }
        } else {
            Err(PdfLoadError::FileError)
        };

        let _ = mp.send(result);
    });
}

fn handle_file_load_error(
    app: &mut PdfCoordPickerApp,
    ctx: &egui::Context,
    ui: &mut egui::Ui,
    ){
            match app.receiver.try_recv() {
                Ok(Ok((path, page_images))) => {
                    app.pdf_file_path = Some(path);
                    app.waiting_for_file = false;
                    app.init_pdf_page_images(ctx, page_images)
                }
                //TODO: ui elements need some file load state to be actually displayed for
                //longer
                Ok(Err(PdfLoadError::FileError)) => {
                    app.waiting_for_file = false;
                    ui.label("Could not open file");
                }
                Ok(Err(PdfLoadError::PdfError((path, e)))) => {
                    app.waiting_for_file = false;
                    ui.label(format!(
                        "Could not load file='{}'. Pdf load error: {}",
                        path.to_string_lossy(),
                        e
                    ));
                }
                Err(TryRecvError::Empty) => {
                    ui.spinner();
                }
                Err(TryRecvError::Disconnected) => {
                    app.waiting_for_file = false;
                    ui.label(format!("Error: Connection to file dialog was lost."));
                }
            }
}


