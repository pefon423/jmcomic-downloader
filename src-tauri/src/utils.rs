use std::{
    collections::{HashMap, HashSet},
    io::Read,
    path::{Path, PathBuf},
};

use eyre::{OptionExt, WrapErr};
use tauri::AppHandle;
use tracing::instrument;
use walkdir::WalkDir;

use crate::{
    extensions::{AppHandleExt, EyreReportToMessage, WalkDirEntryExt},
    types::Comic,
};

pub fn filename_filter(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '\\' | '/' | '\n' => ' ',
            ':' => '：',
            '*' => '⭐',
            '?' => '？',
            '"' => '\'',
            '<' => '《',
            '>' => '》',
            '|' => '丨',
            _ => c,
        })
        .collect::<String>()
        .trim()
        .trim_end_matches('.')
        .trim()
        .to_string()
}

// 计算MD5哈希并返回十六进制字符串
pub fn md5_hex(data: &str) -> String {
    format!("{:x}", md5::compute(data))
}

#[instrument(level = "error", skip_all)]
pub fn create_id_to_dir_map(app: &AppHandle) -> eyre::Result<HashMap<i64, PathBuf>> {
    let mut id_to_dir_map: HashMap<i64, PathBuf> = HashMap::new();
    let mut descriptions: Vec<(i64, String)> = Vec::new();
    let download_dir = app.get_config().read().download_dir.clone();
    if !download_dir.exists() {
        return Ok(id_to_dir_map);
    }

    for entry in WalkDir::new(&download_dir)
        .into_iter()
        .filter_map(Result::ok)
    {
        let path = entry.path();
        if !entry.is_comic_metadata() {
            continue;
        }

        let metadata_str =
            std::fs::read_to_string(path).wrap_err(format!("读取`{}`失败", path.display()))?;
        let comic_json: serde_json::Value = serde_json::from_str(&metadata_str).wrap_err(
            format!("将`{}`反序列化为serde_json::Value失败", path.display()),
        )?;
        let id = comic_json
            .get("id")
            .and_then(serde_json::Value::as_i64)
            .ok_or_eyre(format!("`{}`没有`id`字段", path.display()))?;

        let parent = path
            .parent()
            .ok_or_eyre(format!("`{}`没有父目录", path.display()))?;

        id_to_dir_map.entry(id).or_insert(parent.to_path_buf());

        // 顺便把本地元数据里的描述收进缓存
        if let Some(description) = comic_json
            .get("description")
            .and_then(serde_json::Value::as_str)
        {
            descriptions.push((id, description.to_string()));
        }
    }
    cache_descriptions(app, descriptions);
    Ok(id_to_dir_map)
}

const DESCRIPTION_CACHE_FILENAME: &str = "描述快取.json";

static DESCRIPTION_CACHE: std::sync::OnceLock<std::sync::Mutex<HashMap<i64, String>>> =
    std::sync::OnceLock::new();

fn description_cache(app: &AppHandle) -> &'static std::sync::Mutex<HashMap<i64, String>> {
    use tauri::Manager;
    DESCRIPTION_CACHE.get_or_init(|| {
        let map = app
            .path()
            .app_data_dir()
            .ok()
            .and_then(|dir| std::fs::read_to_string(dir.join(DESCRIPTION_CACHE_FILENAME)).ok())
            .and_then(|s| serde_json::from_str(s.trim_start_matches('\u{feff}')).ok())
            .unwrap_or_default();
        std::sync::Mutex::new(map)
    })
}

/// 读取缓存里的漫画描述(空字符串代表确认过没有描述)
pub fn cached_description(app: &AppHandle, comic_id: i64) -> Option<String> {
    description_cache(app).lock().ok()?.get(&comic_id).cloned()
}

/// 把漫画描述存进缓存，有变化时写回app数据目录下的`描述快取.json`
pub fn cache_descriptions<I: IntoIterator<Item = (i64, String)>>(app: &AppHandle, items: I) {
    use tauri::Manager;
    let Ok(mut cache) = description_cache(app).lock() else {
        return;
    };
    let mut changed = false;
    for (id, description) in items {
        if cache.get(&id) != Some(&description) {
            cache.insert(id, description);
            changed = true;
        }
    }
    if !changed {
        return;
    }
    let Ok(dir) = app.path().app_data_dir() else {
        return;
    };
    match serde_json::to_string(&*cache) {
        Ok(json) => {
            if let Err(err) = std::fs::write(dir.join(DESCRIPTION_CACHE_FILENAME), json) {
                tracing::warn!("写入描述缓存失败: {err}");
            }
        }
        Err(err) => tracing::warn!("序列化描述缓存失败: {err}"),
    }
}

/// 在本地（目录或zip）中扫描到的漫画
pub struct ScannedComic {
    pub id: i64,
    pub name: String,
    /// 本地已存在的章节ID
    pub chapter_ids: HashSet<i64>,
}

/// zip中的元数据信息
#[derive(Default)]
pub struct ZipMetadata {
    /// zip中包含漫画元数据时，为(漫画ID, 漫画名)
    pub comic: Option<(i64, String)>,
    pub chapter_ids: HashSet<i64>,
}

fn parse_json_file(path: &Path) -> eyre::Result<serde_json::Value> {
    let json_str =
        std::fs::read_to_string(path).wrap_err(format!("读取`{}`失败", path.display()))?;
    let json_str = json_str.trim_start_matches('\u{feff}');
    serde_json::from_str(json_str).wrap_err(format!("解析`{}`失败", path.display()))
}

/// 读取zip中的漫画元数据和章节元数据，不会解压图片
#[instrument(level = "error", skip_all, fields(zip_path = %zip_path.display()))]
pub fn read_zip_metadata(zip_path: &Path) -> eyre::Result<ZipMetadata> {
    let file = std::fs::File::open(zip_path)
        .wrap_err(format!("打开`{}`失败", zip_path.display()))?;
    let mut archive = zip::ZipArchive::new(file).wrap_err("读取zip失败")?;

    let mut zip_metadata = ZipMetadata::default();
    for i in 0..archive.len() {
        // 先只看文件名(中央目录已在内存)，避免对每张图片都seek到大zip里读取
        let Some(name) = archive.name_for_index(i) else {
            continue;
        };
        if !name.ends_with("元数据.json") {
            continue;
        }

        let mut entry = archive.by_index(i).wrap_err("读取zip条目失败")?;
        if !entry.is_file() {
            continue;
        }

        let entry_name = entry.name().to_string();
        let is_chapter_metadata = entry_name.ends_with("章节元数据.json");
        let is_comic_metadata = !is_chapter_metadata && entry_name.ends_with("元数据.json");
        if !is_chapter_metadata && !is_comic_metadata {
            continue;
        }

        let mut json_str = String::new();
        entry
            .read_to_string(&mut json_str)
            .wrap_err(format!("读取zip中的`{entry_name}`失败"))?;
        let json: serde_json::Value = serde_json::from_str(json_str.trim_start_matches('\u{feff}'))
            .wrap_err(format!("解析zip中的`{entry_name}`失败"))?;

        if is_chapter_metadata {
            if let Some(chapter_id) = json.get("chapterId").and_then(serde_json::Value::as_i64) {
                zip_metadata.chapter_ids.insert(chapter_id);
            }
        } else if zip_metadata.comic.is_none() {
            if let Some(id) = json.get("id").and_then(serde_json::Value::as_i64) {
                let name = json
                    .get("name")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_default()
                    .to_string();
                zip_metadata.comic = Some((id, name));
            }
        }
    }

    Ok(zip_metadata)
}

/// zip索引文件名，放在zip所在目录，记录该目录下各个zip的元数据，避免每次都打开zip
const ZIP_INDEX_FILENAME: &str = "zip索引.json";

#[derive(Default, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct ZipIndexEntry {
    size: u64,
    /// 修改时间(毫秒)
    mtime: i64,
    comic_id: Option<i64>,
    comic_name: Option<String>,
    chapter_ids: Vec<i64>,
}

/// 获取zip的(大小, 修改时间毫秒)，用来判断zip有没有被改动过
fn zip_fingerprint(zip_path: &Path) -> Option<(u64, i64)> {
    let metadata = std::fs::metadata(zip_path).ok()?;
    let mtime = metadata
        .modified()
        .ok()?
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_millis();
    Some((metadata.len(), i64::try_from(mtime).ok()?))
}

/// zip索引的内存缓存，按目录加载，最后统一写回
#[derive(Default)]
pub struct ZipIndexCache {
    /// 目录 -> (文件名 -> 索引项, 是否被修改过)
    dirs: HashMap<PathBuf, (HashMap<String, ZipIndexEntry>, bool)>,
}

impl ZipIndexCache {
    fn dir_index(&mut self, dir: &Path) -> &mut (HashMap<String, ZipIndexEntry>, bool) {
        self.dirs.entry(dir.to_path_buf()).or_insert_with(|| {
            let index = std::fs::read_to_string(dir.join(ZIP_INDEX_FILENAME))
                .ok()
                .and_then(|s| serde_json::from_str(s.trim_start_matches('\u{feff}')).ok())
                .unwrap_or_default();
            (index, false)
        })
    }

    /// 记录zip的元数据(例如刚打包完成时)
    pub fn record(&mut self, zip_path: &Path, zip_metadata: &ZipMetadata) {
        let (Some(dir), Some(name), Some((size, mtime))) = (
            zip_path.parent(),
            zip_path.file_name().map(|n| n.to_string_lossy().to_string()),
            zip_fingerprint(zip_path),
        ) else {
            return;
        };
        let mut chapter_ids: Vec<i64> = zip_metadata.chapter_ids.iter().copied().collect();
        chapter_ids.sort_unstable();
        let entry = ZipIndexEntry {
            size,
            mtime,
            comic_id: zip_metadata.comic.as_ref().map(|(id, _)| *id),
            comic_name: zip_metadata.comic.as_ref().map(|(_, name)| name.clone()),
            chapter_ids,
        };
        let dir_index = self.dir_index(dir);
        dir_index.0.insert(name, entry);
        dir_index.1 = true;
    }

    /// 优先使用索引，大小和修改时间都一致才信任；否则打开zip读取并更新索引
    pub fn get_metadata(&mut self, zip_path: &Path) -> eyre::Result<ZipMetadata> {
        if let (Some(dir), Some(name), Some((size, mtime))) = (
            zip_path.parent(),
            zip_path.file_name().map(|n| n.to_string_lossy().to_string()),
            zip_fingerprint(zip_path),
        ) {
            if let Some(entry) = self.dir_index(dir).0.get(&name) {
                if entry.size == size && entry.mtime == mtime {
                    return Ok(ZipMetadata {
                        comic: entry.comic_id.map(|id| (id, entry.comic_name.clone().unwrap_or_default())),
                        chapter_ids: entry.chapter_ids.iter().copied().collect(),
                    });
                }
            }
        }
        let zip_metadata = read_zip_metadata(zip_path)?;
        self.record(zip_path, &zip_metadata);
        Ok(zip_metadata)
    }

    /// 把有改动的索引写回磁盘
    pub fn flush(&mut self) {
        for (dir, (index, dirty)) in &mut self.dirs {
            if !*dirty {
                continue;
            }
            let path = dir.join(ZIP_INDEX_FILENAME);
            match serde_json::to_string_pretty(index) {
                Ok(json) => {
                    if let Err(err) = std::fs::write(&path, json) {
                        tracing::warn!("写入`{}`失败: {err}", path.display());
                        continue;
                    }
                    *dirty = false;
                }
                Err(err) => tracing::warn!("序列化`{}`失败: {err}", path.display()),
            }
        }
    }
}

/// 扫描下载目录，找出所有本地已有的漫画及其章节，章节可以是目录，也可以是zip，
/// 漫画本身也可以整个被打包进一个zip
#[instrument(level = "error", skip_all)]
pub fn scan_local_comics(app: &AppHandle) -> Vec<ScannedComic> {
    let download_dir = app.get_config().read().download_dir.clone();
    if !download_dir.exists() {
        return Vec::new();
    }

    let mut comics: HashMap<i64, ScannedComic> = HashMap::new();
    // 漫画目录 -> 漫画ID
    let mut comic_dirs: HashMap<PathBuf, i64> = HashMap::new();
    // (章节所在位置的目录, 章节ID)，最终要归属到祖先中的某个漫画目录
    let mut chapters: Vec<(PathBuf, i64)> = Vec::new();
    let mut zip_index = ZipIndexCache::default();

    for entry in WalkDir::new(&download_dir)
        .into_iter()
        .filter_map(Result::ok)
    {
        let path = entry.path();
        let Some(parent) = path.parent() else {
            continue;
        };

        if entry.is_comic_metadata() {
            let json = match parse_json_file(path) {
                Ok(json) => json,
                Err(err) => {
                    tracing::warn!("{}", err.to_message());
                    continue;
                }
            };
            let Some(id) = json.get("id").and_then(serde_json::Value::as_i64) else {
                continue;
            };
            let name = json
                .get("name")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default()
                .to_string();
            comic_dirs.insert(parent.to_path_buf(), id);
            comics.entry(id).or_insert_with(|| ScannedComic {
                id,
                name,
                chapter_ids: HashSet::new(),
            });
        } else if entry.is_chapter_metadata() {
            let json = match parse_json_file(path) {
                Ok(json) => json,
                Err(err) => {
                    tracing::warn!("{}", err.to_message());
                    continue;
                }
            };
            if let Some(chapter_id) = json.get("chapterId").and_then(serde_json::Value::as_i64) {
                chapters.push((parent.to_path_buf(), chapter_id));
            }
        } else if entry.is_zip() {
            let zip_metadata = match zip_index.get_metadata(path) {
                Ok(zip_metadata) => zip_metadata,
                Err(err) => {
                    tracing::warn!("{}", err.to_message());
                    continue;
                }
            };
            if let Some((id, name)) = zip_metadata.comic {
                // 整个漫画被打包成了zip
                comics
                    .entry(id)
                    .or_insert_with(|| ScannedComic {
                        id,
                        name,
                        chapter_ids: HashSet::new(),
                    })
                    .chapter_ids
                    .extend(zip_metadata.chapter_ids);
            } else {
                // 只有一个章节被打包成了zip
                chapters.extend(
                    zip_metadata
                        .chapter_ids
                        .into_iter()
                        .map(|chapter_id| (parent.to_path_buf(), chapter_id)),
                );
            }
        }
    }

    zip_index.flush();

    for (location, chapter_id) in chapters {
        let comic_id = location
            .ancestors()
            .find_map(|ancestor| comic_dirs.get(ancestor));
        if let Some(comic) = comic_id.and_then(|id| comics.get_mut(id)) {
            comic.chapter_ids.insert(chapter_id);
        }
    }

    comics.into_values().collect()
}

#[instrument(level = "error", skip_all, fields(aid = aid))]
pub async fn get_comic(app: AppHandle, aid: i64) -> eyre::Result<Comic> {
    let jm_client = app.get_jm_client();

    let comic_resp_data = jm_client.get_comic(aid).await?;

    let comic = Comic::from_comic_resp_data(&app, comic_resp_data)?;

    Ok(comic)
}
