use std::{
    ops::Range,
    path::{Path, PathBuf},
};

use gpui::InteractiveElement as _;
use gpui::ListHorizontalSizingBehavior;
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{InteractiveElementExt, h_flex, v_flex};

use crate::layout::{fs_data::FsNode, right_panel::FileListState};

#[derive(Clone)]
struct ExplorerNode {
    path: PathBuf,
    label: SharedString,
    is_dir: bool,
    expanded: bool,
    children_loaded: bool,
    children: Vec<ExplorerNode>,
}

#[derive(Clone)]
struct VisibleEntry {
    path: PathBuf,
    label: SharedString,
    is_dir: bool,
    expanded: bool,
    depth: usize,
    selected: bool,
}

pub struct ExplorerState {
    roots: Vec<ExplorerNode>,
    selected_path: Option<PathBuf>,
    list_state: Entity<FileListState>,
}

impl ExplorerState {
    pub fn new(start_dir: PathBuf, list_state: Entity<FileListState>) -> Self {
        let mut state = Self {
            roots: Vec::new(),
            selected_path: Some(start_dir.clone()),
            list_state,
        };
        state.rebuild_roots(&start_dir);
        state
    }

    fn toggle_expand(&mut self, path: &Path, cx: &mut Context<Self>) {
        if toggle_expand_in(&mut self.roots, path) {
            cx.notify();
        }
    }

    fn select_path(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        if !path.is_dir() {
            self.show_file_in_list(path, cx);
            return;
        }

        self.selected_path = Some(path.clone());
        self.ensure_visible(&path);
        cx.notify();
    }

    pub fn activate_path(&mut self, path: &Path, cx: &mut Context<Self>) {
        let clicked_path = path.to_path_buf();
        let target_dir = if path.is_dir() {
            clicked_path.clone()
        } else {
            path.parent()
                .map(Path::to_path_buf)
                .unwrap_or(clicked_path.clone())
        };
        let rows = super::fs_data::file_rows_for(&target_dir);
        let selected_file = (!path.is_dir()).then_some(clicked_path.as_path());

        self.list_state.update(cx, |state, cx| {
            state.set_directory(target_dir.clone(), rows, selected_file, cx);
        });
        self.rebuild_roots(&target_dir);
        self.selected_path = Some(clicked_path.clone());
        self.ensure_visible(&clicked_path);
        cx.notify();
    }

    pub fn navigate_up(&mut self, cx: &mut Context<Self>) {
        let current_dir = self.list_state.read(cx).current_dir_path().to_path_buf();

        if let Some(parent) = current_dir.parent() {
            self.activate_path(parent, cx);
        }
    }

    pub fn can_navigate_up(&self, cx: &App) -> bool {
        self.list_state
            .read(cx)
            .current_dir_path()
            .parent()
            .is_some()
    }

    fn show_file_in_list(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        let Some(parent_dir) = path.parent().map(Path::to_path_buf) else {
            return;
        };
        let rows = super::fs_data::file_rows_for(&parent_dir);

        self.list_state.update(cx, |state, cx| {
            state.set_directory(parent_dir.clone(), rows, Some(path.as_path()), cx);
        });
        self.selected_path = Some(path.clone());
        self.ensure_visible(&path);
        cx.notify();
    }

    fn visible_entries(&self) -> Vec<VisibleEntry> {
        let mut entries = Vec::new();
        for root in &self.roots {
            root.collect_visible(0, self.selected_path.as_deref(), &mut entries);
        }
        entries
    }

    fn ensure_visible(&mut self, path: &Path) {
        ensure_path_loaded(&mut self.roots, path);
    }

    fn rebuild_roots(&mut self, root_path: &Path) {
        let mut root = ExplorerNode::from_fs(
            super::fs_data::node_for_path(root_path).expect("failed to open explorer root"),
        );
        root.expanded = true;
        root.load_children();
        self.roots = vec![root];
    }
}

impl Render for ExplorerState {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = cx.entity();
        let entries = self.visible_entries();

        div().size_full().relative().child(
            uniform_list(
                "explorer-entries",
                entries.len(),
                cx.processor(move |_this, range: Range<usize>, _window, _cx| {
                    let mut items = Vec::with_capacity(range.len());

                    for ix in range {
                        let entry = entries[ix].clone();
                        let row_path = entry.path.clone();
                        let toggle_path = entry.path.clone();
                        let row_id: SharedString =
                            format!("explorer-row-{}", row_path.display()).into();
                        let toggle_id: SharedString =
                            format!("explorer-toggle-{}", toggle_path.display()).into();

                        let item = h_flex()
                            .id(row_id)
                            .h(px(28.))
                            .w_full()
                            .items_center()
                            .gap_1()
                            .px_2()
                            .border_1()
                            .border_color(rgba(0x00000000))
                            .when(!entry.selected, |this| {
                                this.hover(|this| this.bg(rgb(0xf1f5f9)))
                            })
                            .when(entry.selected, |this| {
                                this.bg(rgb(0xe0f2fe)).border_color(rgb(0x7dd3fc))
                            })
                            .on_click({
                                let state = state.clone();
                                let row_path = row_path.clone();
                                move |_, _, app| {
                                    state.update(app, |this, cx| {
                                        this.select_path(row_path.clone(), cx);
                                    });
                                }
                            })
                            .on_double_click({
                                let state = state.clone();
                                let row_path = row_path.clone();
                                move |_, _, app| {
                                    state.update(app, |this, cx| this.activate_path(&row_path, cx));
                                }
                            })
                            .child(
                                h_flex()
                                    .id(toggle_id)
                                    .ml(px(entry.depth as f32 * 14.))
                                    .w(px(18.))
                                    .justify_center()
                                    .when(entry.is_dir, |this| {
                                        this.on_click({
                                            let state = state.clone();
                                            let toggle_path = toggle_path.clone();
                                            move |_, _, app| {
                                                app.stop_propagation();
                                                state.update(app, |this, cx| {
                                                    this.toggle_expand(&toggle_path, cx)
                                                });
                                            }
                                        })
                                        .child(if entry.expanded { "▾" } else { "▸" })
                                    })
                                    .when(!entry.is_dir, |this| this.child("")),
                            )
                            .child(div().text_sm().child(entry.label))
                            .child(div().flex_1());

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

pub fn left_panel(state: &Entity<ExplorerState>) -> impl IntoElement {
    v_flex()
        .flex_1()
        .h_full()
        .p_1()
        .bg(rgb(0xffffff))
        .overflow_hidden()
        .child(state.clone())
}

impl ExplorerNode {
    fn from_fs(node: FsNode) -> Self {
        Self {
            path: node.path,
            label: node.label,
            is_dir: node.is_dir,
            expanded: false,
            children_loaded: false,
            children: Vec::new(),
        }
    }

    fn load_children(&mut self) {
        if !self.is_dir || self.children_loaded {
            return;
        }

        self.children = super::fs_data::directory_children(&self.path)
            .into_iter()
            .map(ExplorerNode::from_fs)
            .collect();
        self.children_loaded = true;
    }

    fn collect_visible(
        &self,
        depth: usize,
        selected_path: Option<&Path>,
        entries: &mut Vec<VisibleEntry>,
    ) {
        entries.push(VisibleEntry {
            path: self.path.clone(),
            label: self.label.clone(),
            is_dir: self.is_dir,
            expanded: self.expanded,
            depth,
            selected: selected_path.is_some_and(|path| path == self.path.as_path()),
        });

        if self.is_dir && self.expanded {
            for child in &self.children {
                child.collect_visible(depth + 1, selected_path, entries);
            }
        }
    }
}

fn toggle_expand_in(nodes: &mut [ExplorerNode], path: &Path) -> bool {
    for node in nodes {
        if node.path == path {
            if node.is_dir {
                if !node.expanded {
                    node.load_children();
                }
                node.expanded = !node.expanded;
            }
            return true;
        }

        if toggle_expand_in(&mut node.children, path) {
            return true;
        }
    }

    false
}

fn ensure_path_loaded(nodes: &mut [ExplorerNode], path: &Path) -> bool {
    for node in nodes {
        if path.starts_with(&node.path) {
            if node.is_dir {
                node.load_children();
                node.expanded = true;
            }
            if node.path == path {
                return true;
            }
            if ensure_path_loaded(&mut node.children, path) {
                return true;
            }
        }
    }

    false
}
