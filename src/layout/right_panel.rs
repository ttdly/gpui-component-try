use std::path::{Path, PathBuf};

use gpui::InteractiveElement as _;
use gpui::*;
use gpui_component::{
    Icon, IconName, IndexPath, Sizable, StyledExt, h_flex,
    list::{List, ListDelegate, ListItem, ListState},
    tag::Tag,
    v_flex,
};

use crate::layout::left_panel::ExplorerState;

#[derive(Clone)]
pub struct FileRow {
    path: PathBuf,
    name: SharedString,
    kind: SharedString,
    size: SharedString,
    is_dir: bool,
}

impl FileRow {
    pub fn new(
        path: PathBuf,
        name: impl Into<SharedString>,
        kind: impl Into<SharedString>,
        size: impl Into<SharedString>,
        is_dir: bool,
    ) -> Self {
        Self {
            path,
            name: name.into(),
            kind: kind.into(),
            size: size.into(),
            is_dir,
        }
    }

    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn is_dir(&self) -> bool {
        self.is_dir
    }
}

pub struct FileListDelegate {
    current_dir: PathBuf,
    rows: Vec<FileRow>,
    selected: Option<IndexPath>,
    explorer: Option<Entity<ExplorerState>>,
}

impl FileListDelegate {
    pub fn new(current_dir: PathBuf, rows: Vec<FileRow>) -> Self {
        Self {
            current_dir,
            rows,
            selected: Some(IndexPath::new(0)),
            explorer: None,
        }
    }

    pub fn bind_explorer(&mut self, explorer: Entity<ExplorerState>) {
        self.explorer = Some(explorer);
    }

    pub fn set_directory(
        &mut self,
        current_dir: PathBuf,
        rows: Vec<FileRow>,
        selected_path: Option<&Path>,
    ) {
        self.current_dir = current_dir;
        self.rows = rows;
        self.selected = selected_path
            .and_then(|selected_path| {
                self.rows
                    .iter()
                    .position(|row| row.path() == selected_path)
                    .map(IndexPath::new)
            })
            .or_else(|| (!self.rows.is_empty()).then(|| IndexPath::new(0)));
    }

    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn current_dir_label(&self) -> String {
        relative_label(&self.current_dir)
    }

    pub fn current_dir_path(&self) -> &Path {
        &self.current_dir
    }
}

impl ListDelegate for FileListDelegate {
    type Item = ListItem;

    fn items_count(&self, _: usize, _: &App) -> usize {
        self.rows.len()
    }

    fn render_item(
        &mut self,
        ix: IndexPath,
        _: &mut Window,
        _: &mut Context<ListState<Self>>,
    ) -> Option<Self::Item> {
        let row = self.rows.get(ix.row)?;
        let selected = self
            .selected
            .is_some_and(|selected_ix| selected_ix.eq_row(ix));
        let explorer = self.explorer.clone();
        let row_path = row.path().to_path_buf();
        let content_id: SharedString = format!("file-row-content-{}", row_path.display()).into();

        Some(
            ListItem::new(ix).child(
                h_flex()
                    .id(content_id)
                    .w_full()
                    .items_center()
                    .justify_between()
                    .gap_3()
                    .rounded_md()
                    .border_1()
                    .border_color(if selected {
                        rgb(0x7dd3fc)
                    } else {
                        rgba(0x00000000)
                    })
                    .bg(if selected {
                        rgb(0xe0f2fe)
                    } else {
                        rgba(0x00000000)
                    })
                    .px_3()
                    .py_2()
                    .on_click(move |event, _, app| {
                        if let Some(explorer) = explorer.clone() {
                            if event.click_count() == 2 && row_path.is_dir() {
                                app.stop_propagation();
                                explorer.update(app, |this, cx| this.activate_path(&row_path, cx));
                            }
                        }
                    })
                    .child(
                        h_flex().items_center().gap_3().child(file_icon(row)).child(
                            v_flex()
                                .gap_1()
                                .child(div().font_medium().child(row.name.clone()))
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(rgb(0x64748b))
                                        .child(relative_name(row)),
                                ),
                        ),
                    )
                    .child(
                        h_flex()
                            .items_center()
                            .gap_3()
                            .child(file_kind_tag(row))
                            .child(
                                div()
                                    .min_w(px(56.))
                                    .text_right()
                                    .text_sm()
                                    .text_color(rgb(0x64748b))
                                    .child(row.size.clone()),
                            ),
                    ),
            ),
        )
    }

    fn set_selected_index(
        &mut self,
        ix: Option<IndexPath>,
        _: &mut Window,
        cx: &mut Context<ListState<Self>>,
    ) {
        self.selected = ix;
        cx.notify();
    }
}

pub fn right_panel(
    state: &Entity<ListState<FileListDelegate>>,
    _explorer: &Entity<ExplorerState>,
    _current_dir_label: &str,
    _item_count: usize,
) -> impl IntoElement {
    v_flex()
        .flex_1()
        .h_full()
        .pt_1()
        .pb_1()
        .border_r_1()
        .border_color(rgb(0xe4e4e7))
        .bg(rgb(0xffffff))
        .overflow_hidden()
        .gap_2()
        .child(List::new(state).size_full().px_2().py_2())
}

fn file_icon(row: &FileRow) -> impl IntoElement {
    let icon = if row.is_dir() {
        IconName::Folder
    } else {
        IconName::File
    };
    let color = if row.is_dir() {
        rgb(0xd97706)
    } else {
        rgb(0x0284c7)
    };

    Icon::new(icon).small().text_color(color)
}

fn file_kind_tag(row: &FileRow) -> impl IntoElement {
    let tag = if row.is_dir() {
        Tag::warning()
    } else {
        Tag::secondary()
    };

    tag.small().child(row.kind.clone())
}

fn relative_name(row: &FileRow) -> String {
    row.path()
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_string()
}

fn relative_label(path: &Path) -> String {
    if path.parent().is_none() {
        "/".to_string()
    } else {
        path.to_string_lossy().to_string()
    }
}
