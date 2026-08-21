use crate::core::fs::{DateStyle, MY_PC_PATH};
use crate::gui::theme::{ThemeMode, ThemePalette, get_default_palette};
use crate::gui::windows::containers::enums::ItemViewerHeaderColumn;
use crate::gui::windows::structs::AppSettings;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub const CONFIG_VERSION: u32 = 1;
pub fn default_config_version() -> u32 {
    CONFIG_VERSION
}

#[derive(Debug)]
pub enum SettingsError {
    Io(io::Error),
    Parse(String),
    UnsupportedVersion(u32),
}
impl From<io::Error> for SettingsError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}
impl std::fmt::Display for SettingsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "I/O error: {e}"),
            Self::Parse(e) => write!(f, "parse error: {e}"),
            Self::UnsupportedVersion(v) => write!(f, "unsupported version {v}"),
        }
    }
}
impl std::error::Error for SettingsError {}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TagGroupSnapshot {
    pub id: u64,
    pub name: String,
    pub color: [u8; 4],
    pub items: Vec<PathBuf>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TagsSnapshot {
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(default = "default_next_tag_group_id")]
    pub next_group_id: u64,
    #[serde(default)]
    pub groups: Vec<TagGroupSnapshot>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
struct FavoritesDocument {
    #[serde(default = "default_version")]
    version: u32,
    #[serde(default)]
    favorites: Vec<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
struct TagsDocument {
    #[serde(default = "default_version")]
    version: u32,
    #[serde(default = "default_next_tag_group_id")]
    next_group_id: u64,
    #[serde(default)]
    groups: Vec<TagGroupSnapshot>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct BinarySettingsSnapshot {
    folder_scanning_enabled: bool,
    #[serde(default = "default_true")]
    show_hidden_files_folders: bool,
    #[serde(default = "default_true")]
    show_item_viewer_icons: bool,
    #[serde(default)]
    windows_context_menu_enabled: bool,
    window_size_mode: WindowSizeMode,
    start_path: Option<PathBuf>,
    theme: Option<String>,
    #[serde(default)]
    pinned_tabs: Vec<PathBuf>,
    #[serde(default)]
    time_format_24h: bool,
    #[serde(default = "default_date_style")]
    date_style: DateStyle,
    #[serde(default = "default_sort_column")]
    sort_column: crate::gui::utils::SortColumn,
    #[serde(default)]
    sort_ascending: bool,
    #[serde(default = "default_language")]
    language: String,
    #[serde(default = "default_file_order")]
    item_viewer_file_column_order: Vec<ItemViewerHeaderColumn>,
    #[serde(default = "default_drive_order")]
    item_viewer_drive_column_order: Vec<ItemViewerHeaderColumn>,
    #[serde(default = "default_recycle_order")]
    recycle_bin_column_order: Vec<ItemViewerHeaderColumn>,
    #[serde(default = "default_file_sizes")]
    item_viewer_file_column_sizes: Vec<f32>,
    #[serde(default = "default_drive_sizes")]
    item_viewer_drive_column_sizes: Vec<f32>,
    #[serde(default = "default_recycle_sizes")]
    recycle_bin_column_sizes: Vec<f32>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
struct LegacySettingsSnapshot {
    folder_scanning_enabled: bool,
    #[serde(default)]
    windows_context_menu_enabled: bool,
    window_size_mode: LegacyWindowSizeMode,
    start_path: Option<PathBuf>,
    theme: Option<String>,
    #[serde(default)]
    pinned_tabs: Vec<PathBuf>,
    #[serde(default)]
    time_format_24h: bool,
    #[serde(default = "default_sort_column")]
    sort_column: crate::gui::utils::SortColumn,
    #[serde(default)]
    sort_ascending: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
struct BinaryFavoritesSnapshot {
    favorites: Vec<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
struct BinaryTagsSnapshot {
    #[serde(default = "default_next_tag_group_id")]
    next_group_id: u64,
    #[serde(default)]
    groups: Vec<TagGroupSnapshot>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
struct BinaryThemeSnapshot {
    version: u32,
    light: ThemePalette,
    dark: ThemePalette,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum WindowSizeMode {
    FullScreen,
    Custom { width: f32, height: f32 },
}
#[derive(Clone, Debug, Serialize, Deserialize)]
enum LegacyWindowSizeMode {
    FullScreen,
    HalfScreen,
    Custom { width: f32, height: f32 },
}
impl From<LegacyWindowSizeMode> for WindowSizeMode {
    fn from(v: LegacyWindowSizeMode) -> Self {
        match v {
            LegacyWindowSizeMode::FullScreen => Self::FullScreen,
            LegacyWindowSizeMode::HalfScreen => Self::Custom {
                width: 960.0,
                height: 540.0,
            },
            LegacyWindowSizeMode::Custom { width, height } => Self::Custom { width, height },
        }
    }
}
impl Default for WindowSizeMode {
    fn default() -> Self {
        Self::Custom {
            width: 1200.0,
            height: 800.0,
        }
    }
}

fn default_version() -> u32 {
    CONFIG_VERSION
}
fn default_true() -> bool {
    true
}
fn default_next_tag_group_id() -> u64 {
    1
}
fn default_language() -> String {
    "en-US".into()
}
fn default_sort_column() -> crate::gui::utils::SortColumn {
    crate::gui::utils::SortColumn::Name
}
fn default_date_style() -> DateStyle {
    DateStyle::UsShort
}
fn default_file_order() -> Vec<ItemViewerHeaderColumn> {
    vec![
        ItemViewerHeaderColumn::Type,
        ItemViewerHeaderColumn::Size,
        ItemViewerHeaderColumn::Modified,
        ItemViewerHeaderColumn::Created,
    ]
}
fn default_drive_order() -> Vec<ItemViewerHeaderColumn> {
    vec![
        ItemViewerHeaderColumn::Type,
        ItemViewerHeaderColumn::Size,
        ItemViewerHeaderColumn::Usage,
    ]
}
fn default_recycle_order() -> Vec<ItemViewerHeaderColumn> {
    vec![
        ItemViewerHeaderColumn::Type,
        ItemViewerHeaderColumn::Size,
        ItemViewerHeaderColumn::Deleted,
        ItemViewerHeaderColumn::Created,
    ]
}
pub fn default_item_viewer_file_column_size() -> Vec<f32> {
    vec![180.0, 60.0, 75.0, 100.0, 100.0]
}
pub fn default_item_viewer_drive_column_size() -> Vec<f32> {
    vec![180.0, 120.0, 150.0]
}
pub fn default_recycle_bin_column_size() -> Vec<f32> {
    vec![180.0, 60.0, 120.0, 100.0, 100.0]
}
fn default_file_sizes() -> Vec<f32> {
    default_item_viewer_file_column_size()
}
fn default_drive_sizes() -> Vec<f32> {
    default_item_viewer_drive_column_size()
}
fn default_recycle_sizes() -> Vec<f32> {
    default_recycle_bin_column_size()
}

fn base_dir() -> Option<PathBuf> {
    dirs::data_local_dir().map(|p| p.join("ExplorerEden"))
}
fn path(name: &str) -> Option<PathBuf> {
    base_dir().map(|p| p.join(name))
}
fn settings_path() -> Option<PathBuf> {
    path("settings.toml")
}
fn theme_path() -> Option<PathBuf> {
    path("theme.toml")
}
fn tags_path() -> Option<PathBuf> {
    path("tags.toml")
}
fn binary_path(name: &str) -> Option<PathBuf> {
    path(name)
}
fn favorites_path(drive: char) -> Option<PathBuf> {
    base_dir().map(|p| p.join("favorites").join(format!("drive_{drive}.toml")))
}
fn old_favorites_path(drive: char) -> Option<PathBuf> {
    base_dir().map(|p| p.join("favorites").join(format!("drive_{drive}.bin")))
}

fn atomic_write(path: &Path, contents: &[u8]) -> Result<(), SettingsError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension(format!(
        "{}.tmp",
        path.extension().and_then(|e| e.to_str()).unwrap_or("toml")
    ));
    fs::write(&tmp, contents)?;
    let old = path.with_extension("toml.old");
    if path.exists() {
        let _ = fs::remove_file(&old);
        fs::rename(path, &old)?;
    }
    match fs::rename(&tmp, path) {
        Ok(()) => {
            let _ = fs::remove_file(&old);
            Ok(())
        }
        Err(error) => {
            let _ = fs::remove_file(path);
            let _ = fs::rename(&old, path);
            let _ = fs::remove_file(&tmp);
            Err(error.into())
        }
    }
}
fn backup(source: &Path) -> Result<(), SettingsError> {
    if !source.exists() {
        return Ok(());
    }
    let backup = PathBuf::from(format!("{}.bak", source.display()));
    if !backup.exists() {
        fs::rename(source, backup)?;
    }
    Ok(())
}
fn version(v: u32) -> Result<(), SettingsError> {
    if v == CONFIG_VERSION {
        Ok(())
    } else {
        Err(SettingsError::UnsupportedVersion(v))
    }
}
fn binary_decode<T: serde::de::DeserializeOwned>(data: &[u8]) -> Result<T, SettingsError> {
    bincode::deserialize(data)
        .or_else(|_| postcard::from_bytes(data))
        .map_err(|e| SettingsError::Parse(e.to_string()))
}
fn read_text<T: serde::de::DeserializeOwned>(p: &Path) -> Result<T, SettingsError> {
    toml::from_str(&fs::read_to_string(p)?).map_err(|e| SettingsError::Parse(e.to_string()))
}

fn settings_from_binary(s: BinarySettingsSnapshot) -> AppSettings {
    AppSettings {
        version: CONFIG_VERSION,
        folder_scanning_enabled: s.folder_scanning_enabled,
        show_hidden_files_folders: s.show_hidden_files_folders,
        show_item_viewer_icons: s.show_item_viewer_icons,
        windows_context_menu_enabled: s.windows_context_menu_enabled,
        start_path: s.start_path.or_else(|| Some(PathBuf::from(MY_PC_PATH))),
        window_size_mode: s.window_size_mode,
        pinned_tabs: s.pinned_tabs,
        time_format_24h: s.time_format_24h,
        date_style: s.date_style,
        sort_column: s.sort_column,
        sort_ascending: s.sort_ascending,
        language: s.language,
        item_viewer_file_column_order: s.item_viewer_file_column_order,
        item_viewer_drive_column_order: s.item_viewer_drive_column_order,
        recycle_bin_column_order: s.recycle_bin_column_order,
        item_viewer_file_column_sizes: s.item_viewer_file_column_sizes,
        item_viewer_drive_column_sizes: s.item_viewer_drive_column_sizes,
        recycle_bin_column_sizes: s.recycle_bin_column_sizes,
        theme: match s.theme.as_deref() {
            Some("light") => ThemeMode::Light,
            _ => ThemeMode::Dark,
        },
    }
}
impl From<LegacySettingsSnapshot> for AppSettings {
    fn from(s: LegacySettingsSnapshot) -> Self {
        let mut d = AppSettings::default();
        d.folder_scanning_enabled = s.folder_scanning_enabled;
        d.windows_context_menu_enabled = s.windows_context_menu_enabled;
        d.window_size_mode = s.window_size_mode.into();
        d.start_path = s.start_path.or(d.start_path);
        d.pinned_tabs = s.pinned_tabs;
        d.time_format_24h = s.time_format_24h;
        d.sort_column = s.sort_column;
        d.sort_ascending = s.sort_ascending;
        d.theme = match s.theme.as_deref() {
            Some("light") => ThemeMode::Light,
            _ => ThemeMode::Dark,
        };
        d
    }
}

pub fn load_app_settings() -> Result<AppSettings, SettingsError> {
    if let Some(p) = settings_path() {
        if p.exists() {
            let s: AppSettings = read_text(&p)?;
            version(s.version)?;
            return Ok(s);
        }
    }
    let Some(p) = binary_path("settings.bin") else {
        return Ok(AppSettings::default());
    };
    if !p.exists() {
        return Ok(AppSettings::default());
    }
    let data = fs::read(&p)?;
    let migrated = binary_decode::<BinarySettingsSnapshot>(&data)
        .map(settings_from_binary)
        .or_else(|_| binary_decode::<LegacySettingsSnapshot>(&data).map(Into::into))?;
    let text =
        toml::to_string_pretty(&migrated).map_err(|e| SettingsError::Parse(e.to_string()))?;
    atomic_write(settings_path().as_ref().unwrap(), text.as_bytes())?;
    backup(&p)?;
    Ok(migrated)
}
pub fn save_app_settings(settings: &AppSettings) -> Result<(), SettingsError> {
    let text = toml::to_string_pretty(settings).map_err(|e| SettingsError::Parse(e.to_string()))?;
    atomic_write(
        settings_path()
            .ok_or_else(|| SettingsError::Parse("no data directory".into()))?
            .as_path(),
        text.as_bytes(),
    )
}
pub fn load_windows_size_mode_on_start() -> WindowSizeMode {
    load_app_settings()
        .map(|s| s.window_size_mode)
        .unwrap_or_default()
}

pub fn load_favorites(drive: char) -> Result<Option<Vec<String>>, SettingsError> {
    let Some(p) = favorites_path(drive) else {
        return Ok(None);
    };
    if p.exists() {
        let d: FavoritesDocument = read_text(&p)?;
        version(d.version)?;
        return Ok(Some(d.favorites));
    }
    let Some(old) = old_favorites_path(drive) else {
        return Ok(None);
    };
    if !old.exists() {
        return Ok(None);
    }
    let d = binary_decode::<BinaryFavoritesSnapshot>(&fs::read(&old)?)?;
    let doc = FavoritesDocument {
        version: CONFIG_VERSION,
        favorites: d.favorites.clone(),
    };
    atomic_write(
        &p,
        toml::to_string_pretty(&doc)
            .map_err(|e| SettingsError::Parse(e.to_string()))?
            .as_bytes(),
    )?;
    backup(&old)?;
    Ok(Some(d.favorites))
}
pub fn save_favorites(drive: char, favorites: &[String]) -> Result<(), SettingsError> {
    let p =
        favorites_path(drive).ok_or_else(|| SettingsError::Parse("no data directory".into()))?;
    let d = FavoritesDocument {
        version: CONFIG_VERSION,
        favorites: favorites.to_vec(),
    };
    atomic_write(
        &p,
        toml::to_string_pretty(&d)
            .map_err(|e| SettingsError::Parse(e.to_string()))?
            .as_bytes(),
    )
}

pub fn load_tags() -> Result<Option<TagsSnapshot>, SettingsError> {
    let Some(p) = tags_path() else {
        return Ok(None);
    };
    if p.exists() {
        let d: TagsDocument = read_text(&p)?;
        version(d.version)?;
        return Ok(Some(TagsSnapshot {
            version: d.version,
            next_group_id: d.next_group_id,
            groups: d.groups,
        }));
    }
    let Some(old) = binary_path("tags.bin") else {
        return Ok(None);
    };
    if !old.exists() {
        return Ok(None);
    }
    let d = binary_decode::<BinaryTagsSnapshot>(&fs::read(&old)?)?;
    let doc = TagsDocument {
        version: CONFIG_VERSION,
        next_group_id: d.next_group_id,
        groups: d.groups.clone(),
    };
    atomic_write(
        &p,
        toml::to_string_pretty(&doc)
            .map_err(|e| SettingsError::Parse(e.to_string()))?
            .as_bytes(),
    )?;
    backup(&old)?;
    Ok(Some(TagsSnapshot {
        version: CONFIG_VERSION,
        next_group_id: d.next_group_id,
        groups: d.groups,
    }))
}
pub fn save_tags(snapshot: &TagsSnapshot) -> Result<(), SettingsError> {
    let p = tags_path().ok_or_else(|| SettingsError::Parse("no data directory".into()))?;
    let d = TagsDocument {
        version: CONFIG_VERSION,
        next_group_id: snapshot.next_group_id,
        groups: snapshot.groups.clone(),
    };
    atomic_write(
        &p,
        toml::to_string_pretty(&d)
            .map_err(|e| SettingsError::Parse(e.to_string()))?
            .as_bytes(),
    )
}

pub fn load_theme_settings() -> Result<Option<(ThemePalette, ThemePalette)>, SettingsError> {
    let Some(p) = theme_path() else {
        return Ok(None);
    };
    if p.exists() {
        let value: toml::Value = read_text(&p)?;
        let v = match value.get("version") {
            None => 1,
            Some(toml::Value::Integer(v)) if *v >= 0 && *v <= u32::MAX as i64 => *v as u32,
            Some(_) => {
                return Err(SettingsError::Parse(
                    "theme version must be a non-negative integer".into(),
                ));
            }
        };
        version(v)?;
        return Ok(Some((
            overlay_palette(ThemeMode::Light, value.get("light"))?,
            overlay_palette(ThemeMode::Dark, value.get("dark"))?,
        )));
    }
    let Some(old) = binary_path("theme.bin") else {
        return Ok(None);
    };
    if !old.exists() {
        return Ok(None);
    }
    let d = binary_decode::<BinaryThemeSnapshot>(&fs::read(&old)?)?;
    if d.version != crate::gui::theme::THEME_VERSION {
        return Err(SettingsError::UnsupportedVersion(d.version));
    }
    let mut root = toml::map::Map::new();
    root.insert("version".into(), toml::Value::Integer(1));
    root.insert(
        "light".into(),
        toml::Value::try_from(&d.light).map_err(|e| SettingsError::Parse(e.to_string()))?,
    );
    root.insert(
        "dark".into(),
        toml::Value::try_from(&d.dark).map_err(|e| SettingsError::Parse(e.to_string()))?,
    );
    atomic_write(
        &p,
        toml::to_string_pretty(&toml::Value::Table(root))
            .map_err(|e| SettingsError::Parse(e.to_string()))?
            .as_bytes(),
    )?;
    backup(&old)?;
    Ok(Some((d.light, d.dark)))
}
fn overlay_palette(
    mode: ThemeMode,
    patch: Option<&toml::Value>,
) -> Result<ThemePalette, SettingsError> {
    let mut base = toml::Value::try_from(get_default_palette(mode))
        .map_err(|e| SettingsError::Parse(e.to_string()))?;
    if let (Some(dst), Some(toml::Value::Table(src))) = (base.as_table_mut(), patch) {
        for (k, v) in src {
            dst.insert(k.clone(), v.clone());
        }
    }
    base.try_into()
        .map_err(|e| SettingsError::Parse(e.to_string()))
}
pub fn save_theme_settings(light: &ThemePalette, dark: &ThemePalette) -> Result<(), SettingsError> {
    let p = theme_path().ok_or_else(|| SettingsError::Parse("no data directory".into()))?;
    let mut root = toml::map::Map::new();
    root.insert("version".into(), toml::Value::Integer(1));
    root.insert(
        "light".into(),
        toml::Value::try_from(light).map_err(|e| SettingsError::Parse(e.to_string()))?,
    );
    root.insert(
        "dark".into(),
        toml::Value::try_from(dark).map_err(|e| SettingsError::Parse(e.to_string()))?,
    );
    atomic_write(
        &p,
        toml::to_string_pretty(&toml::Value::Table(root))
            .map_err(|e| SettingsError::Parse(e.to_string()))?
            .as_bytes(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_defaults_preserve_explicit_values() {
        assert!(
            toml::from_str::<AppSettings>("folder_scanning_enabled = false\nstart_path = false\n")
                .is_err()
        );

        let settings: AppSettings = toml::from_str(
            "folder_scanning_enabled = false\nstart_path = \"C:/work\"\npinned_tabs = []\nitem_viewer_file_column_sizes = []\nunknown = true\n",
        )
        .unwrap();
        assert!(!settings.folder_scanning_enabled);
        assert!(settings.pinned_tabs.is_empty());
        assert!(settings.item_viewer_file_column_sizes.is_empty());
        assert_eq!(settings.version, CONFIG_VERSION);
    }

    #[test]
    fn theme_overlay_uses_defaults_for_missing_palette_fields() {
        let patch: toml::Value = toml::from_str("text_size = 20.0").unwrap();
        let palette = overlay_palette(ThemeMode::Dark, Some(&patch)).unwrap();
        assert_eq!(palette.text_size, 20.0);
        assert_eq!(palette.font_name, "Segoe UI");
    }

    #[test]
    fn missing_document_versions_default_to_one() {
        let favorites: FavoritesDocument = toml::from_str("favorites = []").unwrap();
        let tags: TagsDocument = toml::from_str("groups = []").unwrap();
        assert_eq!(favorites.version, CONFIG_VERSION);
        assert_eq!(tags.version, CONFIG_VERSION);
    }
}
