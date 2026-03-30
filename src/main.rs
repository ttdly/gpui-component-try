mod layout;

use gpui::*;
use gpui_component::{Root, init};
use layout::NarrowTopWideBottom;

fn main() {
    let app = Application::new();

    app.run(|cx| {
        init(cx);
        cx.on_window_closed(|cx| {
            if cx.windows().is_empty() {
                cx.quit();
            }
        })
        .detach();

        cx.spawn(async move |cx| {
            cx.open_window(WindowOptions::default(), |window, cx| {
                let view = cx.new(|cx| NarrowTopWideBottom::new(window, cx));
                cx.new(|cx| Root::new(view, window, cx))
            })
            .expect("failed to open window");
        })
        .detach();
    });
}
