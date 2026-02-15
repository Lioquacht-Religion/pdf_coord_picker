use egui::{
    epaint, Color32, CursorGrab, Painter, PointerButton, Rect, Response, Sense, Stroke, TextEdit, Vec2
};
use serde::{Deserialize, Serialize};

pub struct PdfTableInput {}

#[derive(Debug, Deserialize, Serialize)]
pub struct PdfInputFieldSerde {
    unique_id: String,
    pos_x: f32,
    pos_y: f32,
    width: f32,
    height: f32,
}

enum CursorAction{
    None,
    ResizeNorth,
}

pub struct PdfInputFieldState {
    pub unique_id: String,
    cursor_action: CursorAction,
    pub rect: Rect,
    pub text: String,
}

impl PdfInputFieldState {
    pub fn new(rect: egui::Rect) -> Self {
        Self {
            unique_id: String::new(),
            cursor_action: CursorAction::None,
            rect: rect,
            text: String::new(),
        }
    }
}

pub struct PdfInputField {
    id: egui::Id,
}

impl PdfInputField {
    pub fn new() -> Self {
        Self {
            id: egui::Id::new("PdfInputField"),
        }
    }

    pub fn show(
        &self,
        state: &mut PdfInputFieldState,
        page_resp: &Response,
        painter: &mut Painter,
        ui: &mut egui::Ui,
    ) -> Response {
        let input_resp = self.ui_draw_input_field(state, page_resp, painter, ui);
        Self::ui_resize_control(state, &page_resp, &input_resp);
        input_resp
    }

    fn ui_draw_input_field(
        &self,
        state: &mut PdfInputFieldState,
        page_resp: &Response,
        painter: &mut Painter,
        ui: &mut egui::Ui,
    ) -> Response {
        let translate_field_rect = state
            .rect
            .translate((page_resp.rect.left(), page_resp.rect.top()).into());
        painter.add(epaint::RectShape::stroke(
            translate_field_rect,
            0.0,
            Stroke::new(3., Color32::BLACK),
            egui::StrokeKind::Outside,
        ));
        Self::paint_circle(
            painter,
            translate_field_rect.left(),
            translate_field_rect.top(),
        );
        Self::paint_circle(
            painter,
            translate_field_rect.right(),
            translate_field_rect.top(),
        );
        Self::paint_circle(
            painter,
            translate_field_rect.left(),
            translate_field_rect.bottom(),
        );
        Self::paint_circle(
            painter,
            translate_field_rect.right(),
            translate_field_rect.bottom(),
        );
        ui.place(
            translate_field_rect,
            TextEdit::singleline(&mut state.text)
                .frame(false)
                .text_color(Color32::BLACK)
                .background_color(Color32::TRANSPARENT),
        )
    }
    fn paint_circle(painter: &mut Painter, center_x: f32, center_y: f32) {
        painter.add(epaint::CircleShape::stroke(
            egui::pos2(center_x, center_y),
            5.,
            Stroke::new(0.5, Color32::BLACK),
        ));
    }

    fn ui_resize_control(
        state: &mut PdfInputFieldState, 
        page_resp: &Response,
        input_resp: &Response
    ) {
        match state.cursor_action{
            CursorAction::None => Self::ui_handle_none_cursor_state(state, input_resp),
            CursorAction::ResizeNorth => Self::ui_handle_resize_north_cursor_state(state, page_resp, input_resp),
        }
    }

    fn ui_handle_resize_north_cursor_state(
        state: &mut PdfInputFieldState,
        page_resp: &Response,
        input_resp: &Response
    ) {
            if input_resp.dragged_by(PointerButton::Primary) && let Some(pos) = input_resp.interact_pointer_pos() {
                println!("top before : {}", input_resp.rect.top());
                let pos_y = (pos.y - page_resp.rect.top()).abs();
                let pos_y = pos_y.clamp(0., state.rect.bottom());
                *state.rect.top_mut() = pos_y;
                println!("top after: {}", input_resp.rect.top());
                println!("pos: {pos}");
            }
            else {
                state.cursor_action = CursorAction::None;
            }
    }

    fn ui_handle_none_cursor_state(state: &mut PdfInputFieldState, input_resp: &Response) {
        if input_resp.has_focus() {
            println!("dragging and focus");
            if let Some(pos) = input_resp.interact_pointer_pos() {
                println!("pos: {pos}");
                println!("top : {}", input_resp.rect.top());
                if input_resp.rect.top() <= pos.y && input_resp.rect.top() + 10. >= pos.y {
                    input_resp
                        .clone()
                        .on_hover_cursor(egui::CursorIcon::ResizeNorth);
                    if input_resp.dragged_by(PointerButton::Primary) {
                        let drag_delta = input_resp.drag_delta();
                        println!("drag delta: {drag_delta}");
                        //*state.rect.top_mut() += drag_delta.y * 1.4;
                        state.cursor_action = CursorAction::ResizeNorth;
                    }
                }
            }
        }
    }
}

