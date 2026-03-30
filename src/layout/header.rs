use gpui::*;
use gpui_component::{Icon, IconName, Sizable, StyledExt, h_flex, v_flex};

use crate::layout::left_panel::ExplorerState;

pub fn top_header(
    explorer: &Entity<ExplorerState>,
    current_dir_label: &str,
    can_go_up: bool,
) -> impl IntoElement {
    let explorer = explorer.clone();

    v_flex()
        .w_full()
        .h(px(56.))
        .justify_center()
        .border_b_1()
        .border_color(rgb(0xd4d4d8))
        .pl(px(24.))
        .pr(px(24.))
        .child(
            h_flex().items_center().justify_between().child(
                h_flex()
                    .items_center()
                    .gap_3()
                    .child(
                        h_flex()
                            .id("go-up-button")
                            .items_center()
                            .gap_2()
                            .px_3()
                            .py_1p5()
                            .rounded_md()
                            .border_1()
                            .border_color(rgb(0xe4e4e7))
                            .bg(if can_go_up {
                                rgb(0xffffff)
                            } else {
                                rgb(0xf4f4f5)
                            })
                            .text_color(if can_go_up {
                                rgb(0x0f172a)
                            } else {
                                rgb(0x94a3b8)
                            })
                            .on_click(move |_, _, app| {
                                if can_go_up {
                                    explorer.update(app, |this, cx| this.navigate_up(cx));
                                }
                            })
                            .child(Icon::new(IconName::ArrowUp).small())
                            .child("上一级"),
                    )
                    .child(
                        div()
                            .text_lg()
                            .font_semibold()
                            .child(format!("当前目录 · {}", current_dir_label)),
                    ),
            ),
        )
}
