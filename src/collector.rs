use serde::{Serialize, Deserialize, Deserializer};
use byteorder::*;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Seek, SeekFrom};
use std::mem::size_of;

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct Item {
    #[serde(deserialize_with = "deserialize_null_default")]
    low_time: u32,
    #[serde(deserialize_with = "deserialize_null_default")]
    high_time: u32,
    #[serde(deserialize_with = "deserialize_null_default")]
    low: u32,
    #[serde(deserialize_with = "deserialize_null_default")]
    high: u32,
}

const ITEM_SIZE: i64 = (4 * size_of::<i32>()) as i64;

fn deserialize_null_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    T: Default + Deserialize<'de>,
    D: Deserializer<'de>,
{
    let opt = Option::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}

#[derive(Deserialize, Debug)]
struct Wrapper {
    data: HashMap<u32, Item>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Metadata {
    number_of_entries: u32
}

#[derive(Serialize, Deserialize, Debug)]
struct SystemMetadata {
    total_bytes: u64,
    new_entries: u32,
    per_item: HashMap<u32, Metadata>,
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let body = ureq::get("https://prices.runescape.wiki/api/v1/osrs/latest")
        .set("User-Agent", "Ty Overby <ty-osrs@pre-alpha.com>")
        .call()?
        .into_json::<Wrapper>()?
        .data;

    fs::create_dir_all("data")?;
    let mut metadata: HashMap<u32, Metadata> = Default::default();
    let mut total_bytes = 0u64;
    let mut new_entries = 0;

    for (k, v) in body {
        let mut file = fs::OpenOptions::new().read(true).write(true).create(true).open(format!("data/{}.bin", k))?;
        let length = file.metadata()?.len();
        total_bytes += length;
        let mut number_of_entries = length / (ITEM_SIZE as u64);
        let mut should_write = true;
        if length > 0 {
            let backwards_count = 10.min(length / ITEM_SIZE as u64) as i64;
            file.seek(SeekFrom::End(-ITEM_SIZE * backwards_count))?;

            for _ in 0 .. backwards_count {
                let item = Item {
                    low_time: file.read_u32::<LittleEndian>()?,
                    high_time: file.read_u32::<LittleEndian>()?,
                    low: file.read_u32::<LittleEndian>()?,
                    high: file.read_u32::<LittleEndian>()?,
                };

                if item == v { 
                    should_write = false;
                }
            }
        }

        if should_write {
            number_of_entries += 1;
            new_entries += 1;
            file.write_u32::<LittleEndian>(v.low_time)?;
            file.write_u32::<LittleEndian>(v.high_time)?;
            file.write_u32::<LittleEndian>(v.low)?;
            file.write_u32::<LittleEndian>(v.high)?;
        }
        metadata.insert(k, Metadata {number_of_entries: number_of_entries as u32 });
    }
    let metadata = SystemMetadata {
        per_item: metadata, 
        total_bytes,
        new_entries,
    };
    println!("new entries: {}", new_entries);
    let metadata_file = File::create("./data/metadata")?;
    serde_json::to_writer_pretty(metadata_file, &metadata)?;
    Ok(())
}
