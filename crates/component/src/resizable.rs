//! What this design system paints inside a resize handle.
//!
//! Base owns the band, the cursor and the drag; everything here is appearance.
//! A divider rests as the same hairline it has always been, and answers the
//! pointer with a pill that grows and solidifies as the pointer engages it:
//! available, held, being dragged.

use std::rc::Rc;

use gpui::{
    AnyElement, App, Axis, ElementId, IntoElement, ParentElement as _, Pixels, Styled as _, Window,
    div, prelude::FluentBuilder as _, px, relative,
};
use gpui_base::{
    ResizeHandleContext, ResizeHandleRenderer, ResizeHandleState, Transition, transition,
};

pub use gpui_base::{
    ResizablePanel, ResizablePanelEvent, ResizablePanelGroup, ResizableState, resizable_panel,
};

use crate::theme::ActiveTheme as _;

/// How thick the indicator is across the divider it sits on.
const INDICATOR_THICKNESS: Pixels = px(3.);

/// Create a [`ResizablePanelGroup`] with horizontal resizing.
pub fn h_resizable(id: impl Into<ElementId>) -> ResizablePanelGroup {
    gpui_base::h_resizable(id).with_handle_appearance(resize_handle_appearance())
}

/// Create a [`ResizablePanelGroup`] with vertical resizing.
pub fn v_resizable(id: impl Into<ElementId>) -> ResizablePanelGroup {
    gpui_base::v_resizable(id).with_handle_appearance(resize_handle_appearance())
}

/// This design system's divider appearance, for a handle that base does not
/// already hand it — a dock edge, or a hand-rolled handle in an application.
pub fn resize_handle_appearance() -> ResizeHandleRenderer {
    Rc::new(|handle, window, cx| Some(render_resize_handle(handle, window, cx)))
}

/// How long the indicator is at each level of engagement, and how solid.
///
/// Idle draws nothing. The hairline is the resting appearance of a divider, and
/// a pill on every divider all the time would be noise in a dock that has a
/// dozen of them.
fn indicator(state: ResizeHandleState) -> (Pixels, f32) {
    match state {
        ResizeHandleState::Idle => (px(0.), 0.),
        ResizeHandleState::Hovered => (px(20.), 0.35),
        ResizeHandleState::Pressed => (px(28.), 0.6),
        ResizeHandleState::Dragging => (px(44.), 0.9),
    }
}

/// The hairline, and the indicator riding on it.
pub(crate) fn render_resize_handle(
    handle: &ResizeHandleContext,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let axis = handle.axis();
    let (target_length, target_opacity) = indicator(handle.state());
    let motion = cx.theme().motion_tokens();
    let policy = Transition::new(motion.duration_fast).easing(motion.easing_move.clone());

    // Both values are sampled on every frame the handle is rendered, whatever
    // it is showing. A transition asks for a frame only while it is moving, so
    // a resting handle costs nothing; sampling it only while the pill is up
    // would instead leave the retained value frozen wherever it was when the
    // pill went away, and the next hover would start from there.
    let length = transition(
        "resizable-handle-indicator-length",
        target_length,
        policy.clone(),
        window,
        cx,
    );
    let opacity = transition(
        "resizable-handle-indicator-opacity",
        target_opacity,
        policy,
        window,
        cx,
    );

    div()
        // The handle's border box is one pixel wide and padded, so its content
        // area is zero and a shrinkable child collapses with it.
        .flex_none()
        .relative()
        .bg(cx.theme().border)
        .map(|line| match axis {
            Axis::Horizontal => line.w(px(1.)).h_full(),
            _ => line.h(px(1.)).w_full(),
        })
        .when(length > px(0.5), |line| {
            line.child(
                div()
                    .absolute()
                    .top(relative(0.5))
                    .left(relative(0.5))
                    .rounded(cx.theme().radius_full())
                    .bg(cx.theme().muted_foreground)
                    .opacity(opacity)
                    // Centred on the hairline rather than beside it: the pill is
                    // wider than the line it rides, so half of it overhangs each
                    // side.
                    .map(|pill| match axis {
                        Axis::Horizontal => pill
                            .w(INDICATOR_THICKNESS)
                            .h(length)
                            .ml(INDICATOR_THICKNESS * -0.5)
                            .mt(length * -0.5),
                        _ => pill
                            .h(INDICATOR_THICKNESS)
                            .w(length)
                            .mt(INDICATOR_THICKNESS * -0.5)
                            .ml(length * -0.5),
                    }),
            )
        })
        .into_any_element()
}
