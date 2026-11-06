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
    }
    Ok(id_to_dir_map)
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
            let zip_metadata = match read_zip_metadata(path) {
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
