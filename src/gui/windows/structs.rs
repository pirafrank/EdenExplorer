use crate::core::drives::DriveInfo;
use crate::core::indexer::WindowSizeMode;
use crate::gui::theme::{ThemeMode, ThemePalette};
use crate::gui::utils::SortColumn;
use crate::gui::windows::containers::enums::ItemViewerHeaderColumn;
use crate::gui::windows::containers::structs::FavoriteItem;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{Duration, Instant};

#[derive(Default)]
pub struct AboutWindow {
    pub open: bool,
}

pub struct ThemeCustomizer {
    pub open: bool,
    pub selected_mode: ThemeMode,
    pub light_palette: ThemePalette,
    pub dark_palette: ThemePalette,
}

impl Default for ThemeCustomizer {
    fn default() -> Self {
        Self {
            open: false,
            selected_mode: ThemeMode::Dark,
            light_palette: crate::gui::theme::get_palette(ThemeMode::Light),
            dark_palette: crate::gui::theme::get_palette(ThemeMode::Dark),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Navigation {
    pub current: PathBuf,
    pub back: Vec<PathBuf>,
    pub forward: Vec<PathBuf>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct AppSettings {
    #[serde(default = "crate::core::indexer::default_config_version")]
    pub version: u32,
    pub folder_scanning_enabled: bool,
    pub show_hidden_files_folders: bool,
    pub show_item_viewer_icons: bool,
    pub windows_context_menu_enabled: bool,
    pub start_path: Option<PathBuf>,
    pub window_size_mode: WindowSizeMode,
    pub pinned_tabs: Vec<PathBuf>,
    pub time_format_24h: bool,
    pub date_style: crate::core::fs::DateStyle,
    pub sort_column: SortColumn,
    pub sort_ascending: bool,
    pub language: String,
    pub item_viewer_file_column_order: Vec<ItemViewerHeaderColumn>,
    pub item_viewer_drive_column_order: Vec<ItemViewerHeaderColumn>,
    pub recycle_bin_column_order: Vec<ItemViewerHeaderColumn>,
    pub item_viewer_file_column_sizes: Vec<f32>,
    pub item_viewer_drive_column_sizes: Vec<f32>,
    pub recycle_bin_column_sizes: Vec<f32>,
    pub theme: ThemeMode,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            version: crate::core::indexer::CONFIG_VERSION,
            folder_scanning_enabled: true,
            show_hidden_files_folders: true,
            show_item_viewer_icons: true,
            windows_context_menu_enabled: false,
            start_path: Some(PathBuf::from(crate::core::fs::MY_PC_PATH)),
            window_size_mode: WindowSizeMode::default(),
            pinned_tabs: Vec::new(),
            time_format_24h: false,
            date_style: crate::core::fs::DateStyle::default(),
            sort_column: SortColumn::Name,
            sort_ascending: true,
            language: "en-US".to_string(),
            item_viewer_file_column_order: vec![
                ItemViewerHeaderColumn::Type,
                ItemViewerHeaderColumn::Size,
                ItemViewerHeaderColumn::Modified,
                ItemViewerHeaderColumn::Created,
            ],
            item_viewer_drive_column_order: vec![
                ItemViewerHeaderColumn::Type,
                ItemViewerHeaderColumn::Size,
                ItemViewerHeaderColumn::Usage,
            ],
            recycle_bin_column_order: vec![
                ItemViewerHeaderColumn::Type,
                ItemViewerHeaderColumn::Size,
                ItemViewerHeaderColumn::Deleted,
                ItemViewerHeaderColumn::Created,
            ],
            item_viewer_file_column_sizes:
                crate::core::indexer::default_item_viewer_file_column_size(),
            item_viewer_drive_column_sizes:
                crate::core::indexer::default_item_viewer_drive_column_size(),
            recycle_bin_column_sizes: crate::core::indexer::default_recycle_bin_column_size(),
            theme: ThemeMode::default(),
        }
    }
}

#[derive(Default)]
pub struct SettingsWindow {
    pub open: bool,
    pub current_settings: AppSettings,
    pub show_reset_favorites_confirmation: bool,
}

pub struct SidebarState {
    pub favorites: Vec<FavoriteItem>,
    pub item_clicked: Option<PathBuf>,
    pub dragging_favorite: Option<usize>,
    pub sidebar_default_width: f32,
    pub cached_drives: Vec<DriveInfo>,
    pub last_drive_refresh: Instant,
    pub non_ntfs_popup_path: Option<PathBuf>,
}

impl Default for SidebarState {
    fn default() -> Self {
        let now = Instant::now();
        Self {
            favorites: vec![],
            dragging_favorite: None,
            item_clicked: None,
            sidebar_default_width: 250.0,
            cached_drives: Vec::new(),
            last_drive_refresh: now.checked_sub(Duration::from_secs(60)).unwrap_or(now),
            non_ntfs_popup_path: None,
        }
    }
}
