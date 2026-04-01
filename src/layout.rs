mod fs_data;
mod header;
mod left_panel;
mod right_panel;

use gpui::*;
use gpui_component::{
    resizable::{ResizableState, h_resizable, resizable_panel},
    v_flex,
};

pub struct NarrowTopWideBottom {
    explorer: Entity<left_panel::ExplorerState>,
    list: Entity<right_panel::FileListState>,
    split: Entity<ResizableState>,
}

impl NarrowTopWideBottom {
    pub fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        let start_dir = fs_data::start_dir();
        let rows = fs_data::file_rows_for(&start_dir);

        let list = cx.new(|_| right_panel::FileListState::new(start_dir.clone(), rows));
        let explorer = cx.new(|_| left_panel::ExplorerState::new(start_dir.clone(), list.clone()));
        let split = cx.new(|_| ResizableState::default());
        list.update(cx, |state, _| {
            state.bind_explorer(explorer.clone());
        });
        cx.observe(&list, |_, _, cx| cx.notify()).detach();
        cx.observe(&explorer, |_, _, cx| cx.notify()).detach();

        Self {
            explorer,
            list,
            split,
        }
    }
}

impl Render for NarrowTopWideBottom {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (item_count, current_dir_label) = {
            let list = self.list.read(cx);
            (list.len(), list.current_dir_label())
        };
        let can_go_up = self.explorer.read(cx).can_navigate_up(cx);

        v_flex()
            .size_full()
            .bg(rgb(0xf8fafc))
            .child(header::top_header(
                &self.explorer,
                &current_dir_label,
                can_go_up,
            ))
            .child(
                v_flex().flex_1().w_full().child(
                    h_resizable("explorer-split")
                        .with_state(&self.split)
                        .child(
                            resizable_panel()
                                .size(px(280.))
                                .size_range(px(180.)..px(520.))
                                .child(left_panel::left_panel(&self.explorer)),
                        )
                        .child(resizable_panel().child(right_panel::right_panel(
                            &self.list,
                            &self.explorer,
                            &current_dir_label,
                            item_count,
                        ))),
                ),
            )
    }
}
