use std::{
    ops::Range,
    path::{Path, PathBuf},
};

use gpui::InteractiveElement as _;
use gpui::ListHorizontalSizingBehavior;
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{
    Icon, IconName, InteractiveElementExt, Sizable, StyledExt, h_flex, tag::Tag, v_flex,
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

pub struct FileListState {
    current_dir: PathBuf,
    rows: Vec<FileRow>,
    selected_path: Option<PathBuf>,
    explorer: Option<Entity<ExplorerState>>,
}

impl FileListState {
    pub fn new(current_dir: PathBuf, rows: Vec<FileRow>) -> Self {
        Self {
            current_dir,
            rows,
            selected_path: None,
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
        cx: &mut Context<Self>,
    ) {
        self.current_dir = current_dir;
        self.rows = rows;
        self.selected_path = selected_path.map(Path::to_path_buf);
        cx.notify();
    }

    fn select_path(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        self.selected_path = Some(path);
        cx.notify();
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

impl Render for FileListState {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = cx.entity();
        let rows = self.rows.clone();
        let selected_path = self.selected_path.clone();
        let explorer = self.explorer.clone();

        div().size_full().relative().child(
            uniform_list(
                "file-list-entries",
                rows.len(),
                cx.processor(move |_this, range: Range<usize>, _window, _cx| {
                    let mut items = Vec::with_capacity(range.len());

                    for ix in range {
                        let row = rows[ix].clone();
                        let row_path = row.path().to_path_buf();
                        let is_selected = selected_path
                            .as_ref()
                            .is_some_and(|selected| selected == &row_path);
                        let row_id: SharedString =
                            format!("file-row-{}", row_path.display()).into();

                        let item = h_flex()
                            .id(row_id)
                            .w_full()
                            .items_center()
                            .justify_between()
                            .gap_3()
                            .border_1()
                            .border_color(if is_selected {
                                rgb(0x7dd3fc)
                            } else {
                                rgba(0x00000000)
                            })
                            .bg(if is_selected {
                                rgb(0xe0f2fe)
                            } else {
                                rgba(0x00000000)
                            })
                            .when(!is_selected, |this| {
                                this.hover(|this| this.bg(rgb(0xf1f5f9)))
                            })
                            .px_3()
                            .py_2()
                            .on_click({
                                let state = state.clone();
                                let row_path = row_path.clone();
                                move |_, _, app| {
                                    state.update(app, |this, cx| {
                                        this.select_path(row_path.clone(), cx)
                                    });
                                }
                            })
                            .on_double_click({
                                let explorer = explorer.clone();
                                let row_path = row_path.clone();
                                move |_, _, app| {
                                    if row_path.is_dir() {
                                        if let Some(explorer) = explorer.clone() {
                                            explorer.update(app, |this, cx| {
                                                this.activate_path(&row_path, cx)
                                            });
                                        }
                                    }
                                }
                            })
                            .child(
                                h_flex()
                                    .items_center()
                                    .gap_3()
                                    .child(file_icon(&row))
                                    .child(
                                        v_flex()
                                            .gap_1()
                                            .child(div().font_medium().child(row.name.clone()))
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .text_color(rgb(0x64748b))
                                                    .child(relative_name(&row)),
                                            ),
                                    ),
                            )
                            .child(
                                h_flex()
                                    .items_center()
                                    .gap_3()
                                    .child(file_kind_tag(&row))
                                    .child(
                                        div()
                                            .min_w(px(56.))
                                            .text_right()
                                            .text_sm()
                                            .text_color(rgb(0x64748b))
                                            .child(row.size.clone()),
                                    ),
                            );

                        items.push(item.into_any_element());
                    }

                    items
                }),
            )
            .with_horizontal_sizing_behavior(ListHorizontalSizingBehavior::Unconstrained)
            .size_full(),
        )
    }
}

pub fn right_panel(
    state: &Entity<FileListState>,
    _explorer: &Entity<ExplorerState>,
    _current_dir_label: &str,
    _item_count: usize,
) -> impl IntoElement {
    v_flex()
        .flex_1()
        .h_full()
        .p_1()
        .border_r_1()
        .border_color(rgb(0xe4e4e7))
        .bg(rgb(0xffffff))
        .overflow_hidden()
        .gap_2()
        .child(state.clone())
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
