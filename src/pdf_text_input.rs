use egui::{
    epaint, Color32, Painter, PointerButton, Pos2, Rect, Response, Stroke, TextEdit
};
use serde::{Deserialize, Serialize};

//pub struct PdfTableInput {}

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
    ResizeWest,
    ResizeSouth,
    ResizeEast,
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
            CursorAction::ResizeNorth => Self::ui_handle_resize_cursor_state(
                &mut state.cursor_action, input_resp, page_resp.rect.top(), 
                0., input_resp.rect.bottom(), state.rect.top_mut(),
                |pos| {pos.y}
            ),
            CursorAction::ResizeWest => Self::ui_handle_resize_cursor_state(
                &mut state.cursor_action, input_resp, 
                    page_resp.rect.left(), 0., state.rect.right(), state.rect.left_mut(),  
                |pos| {pos.x}
            ),
            CursorAction::ResizeSouth => Self::ui_handle_resize_cursor_state(
                &mut state.cursor_action, input_resp, 
                page_resp.rect.top(), state.rect.top(), page_resp.rect.bottom(), state.rect.bottom_mut(),  
                |pos| {pos.y}
            ),
            CursorAction::ResizeEast => Self::ui_handle_resize_cursor_state(
                &mut state.cursor_action, input_resp, 
                page_resp.rect.left(), state.rect.left(), page_resp.rect.right(), state.rect.right_mut(),
                |pos| {pos.x}
            ),

        }
    }

   fn ui_handle_resize_cursor_state(
        cursor_action: &mut CursorAction,
        input_resp: &Response,
        page_translate_side_pos: f32,
        min_pos: f32,
        max_pos: f32,
        side_pos: &mut f32,
        pointer_pos_getter: fn(&Pos2) -> f32,
    ) {
            if input_resp.dragged_by(PointerButton::Primary) && let Some(pos) = input_resp.interact_pointer_pos() {
                let pos_y = (pointer_pos_getter(&pos) - page_translate_side_pos).abs();
                let pos_y = pos_y.clamp(min_pos, max_pos);
                *side_pos = pos_y;
            }
            else {
                *cursor_action = CursorAction::None;
            }
    }

    fn ui_handle_none_cursor_state(state: &mut PdfInputFieldState, input_resp: &Response) {
        if input_resp.has_focus() {
            if let Some(pos) = input_resp.interact_pointer_pos() {
                if input_resp.rect.top() <= pos.y && input_resp.rect.top() + 3. >= pos.y {
                    input_resp
                        .clone()
                        .on_hover_cursor(egui::CursorIcon::ResizeNorth);
                    if input_resp.dragged_by(PointerButton::Primary) {
                        state.cursor_action = CursorAction::ResizeNorth;
                    }
                }
                if input_resp.rect.bottom() >= pos.y && input_resp.rect.bottom() - 3. <= pos.y {
                    input_resp
                        .clone()
                        .on_hover_cursor(egui::CursorIcon::ResizeSouth);
                    if input_resp.dragged_by(PointerButton::Primary) {
                        state.cursor_action = CursorAction::ResizeSouth;
                    }
                }
                if input_resp.rect.left() <= pos.x && input_resp.rect.left() + 10. >= pos.x {
                    input_resp
                        .clone()
                        .on_hover_cursor(egui::CursorIcon::ResizeWest);
                    if input_resp.dragged_by(PointerButton::Primary) {
                        state.cursor_action = CursorAction::ResizeWest;
                    }
                }
                if input_resp.rect.right() >= pos.x && input_resp.rect.right() - 10. <= pos.x {
                    input_resp
                        .clone()
                        .on_hover_cursor(egui::CursorIcon::ResizeEast);
                    if input_resp.dragged_by(PointerButton::Primary) {
                        state.cursor_action = CursorAction::ResizeEast;
                    }
                }
            }
        }
    }
}

