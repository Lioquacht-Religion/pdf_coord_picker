// file_dialog_web.rs

use pdfium_render::prelude::PdfiumError;
use pdfium_render::prelude::Pdfium;

use crate::app::{PdfLoadError, create_images_from_pdf};
use crate::{PdfCoordPickerApp, pdf_load};
use std::sync::mpsc::TryRecvError;

pub fn handle_open_file_dialog_web(
    app: &mut PdfCoordPickerApp,
    ctx: &egui::Context,
    ui: &mut egui::Ui,
) {
    use eframe::wasm_bindgen::JsCast;
    use eframe::wasm_bindgen::prelude::*;
    use eframe::web_sys::HtmlButtonElement;
    use web_sys::Blob;

    let document = web_sys::window().unwrap().document().unwrap();

    // get element by id button
    let button: HtmlButtonElement = document
        .get_element_by_id("button")
        .unwrap()
        .dyn_into::<HtmlButtonElement>()
        .unwrap();

    let mp = app.producer.clone();

    let onclick = Closure::<dyn Fn()>::new(move || {
        // Spawn dialog on main thread
        let task = rfd::AsyncFileDialog::new().pick_file();

        let mp = mp.clone();

        // Await somewhere else
        wasm_bindgen_futures::spawn_local(async move {
            let file = task.await;

            let output = web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .get_element_by_id("output")
                .unwrap();

            if let Some(file) = file {
                if let Ok(pdf_document) =
                    pdf_load::load_pdf_web(&Pdfium::default(), Blob::from(file.inner().clone()))
                        .await
                {
                    let result = match create_images_from_pdf(pdf_document) {
                        Ok(pdf_images) => Ok(("".into(), pdf_images)),
                        Err(e) => Err(PdfLoadError::PdfError(("".into(), e))),
                    };
                    let _ = mp.send(result);
                }
                // If you care about wasm support you just read() the file
                // TODO: read data immediately into pdf page images
                let contents = file.read().await;
                output.set_text_content(Some(&format!(
                    "Picked file: {}, loaded {} bytes",
                    file.file_name(),
                    contents.len()
                )));
            } else {
                output.set_text_content(Some("No file picked"));
            }
        });
    })
    .into_js_value();

    // Browsers require [user activation][mdn] to automatically show the file dialog.
    // This tests using a timer to lose transient user activation such that the file
    // dialog is not show automatically and we fall back to the popup.
    //
    // [mdn]: https://developer.mozilla.org/en-US/docs/Web/Security/User_activation
    let button_delay: HtmlButtonElement = document
        .get_element_by_id("button-delay")
        .unwrap()
        .dyn_into::<HtmlButtonElement>()
        .unwrap();

    button.set_onclick(Some(&onclick.as_ref().unchecked_ref()));

    let delay_onclick = Closure::<dyn Fn()>::new(move || {
        let window = web_sys::window().unwrap();
        window
            .set_timeout_with_callback_and_timeout_and_arguments_0(&onclick.unchecked_ref(), 5000)
            .unwrap();
    })
    .into_js_value();

    button_delay.set_onclick(Some(&delay_onclick.as_ref().unchecked_ref()));
}
