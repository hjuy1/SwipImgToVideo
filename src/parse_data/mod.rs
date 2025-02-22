#![allow(unused)]
pub mod simple_rng;
use crate::{
    err_new, err_new_io,
    error::{Kind, Result},
    prelude::RESOURCE,
    Chunk,
};
use chrono::{Local, NaiveDate};
use serde::Serialize;
use std::{
    collections::HashMap,
    fs::{self, File},
    path::{Path, PathBuf},
};
use structs::{CarFile, Char, CharFile, RealName};

const IMG_PATH: &str = "E:/pictures/arknights";
const DATA_PATH: &str = "./data";

fn img_path(name: &str) -> Result<PathBuf> {
    let img_list = ["skin3", "skin2", "skin1", "2", "1"];
    let out = img_list
        .iter()
        .map(|p| {
            let mut path = PathBuf::from(IMG_PATH);
            path.push(format!("{name}_{p}.png"));
            path
        })
        .find(|p| p.exists())
        .unwrap_or({
            let mut path = PathBuf::from(IMG_PATH);
            path.push(format!("{name}_1.png"));
            path
        });
    if out.exists() {
        Ok(out)
    } else {
        Err(err_new!(
            Kind::IoError(std::io::ErrorKind::NotFound),
            &format!("{name} not found")
        ))
    }
}

fn read_json<T>(file_path: &Path) -> Result<Vec<T>>
where
    T: serde::Serialize + serde::de::DeserializeOwned,
{
    let info: Vec<T> = serde_json::from_reader(File::open(file_path).map_err(|e| err_new_io!(e))?)
        .map_err(|e| err_new!(Kind::Other, &e.to_string()))?;
    Ok(info)
}

fn write_json<P: AsRef<Path>, T: Serialize>(save_name: P, data: &T) -> Result<()> {
    let mut data_path = PathBuf::from(DATA_PATH);
    fs::create_dir_all(&data_path).map_err(|e| err_new_io!(e))?;
    data_path.push(save_name.as_ref());
    let writer = File::create(data_path).map_err(|e| err_new_io!(e))?;
    serde_json::to_writer_pretty(writer, data)
        .map_err(|e| err_new!(Kind::Other, &e.to_string()))?;
    Ok(())
}

pub fn no_skin() -> Result<()> {
    let char_file = Path::new(RESOURCE).join("char/Char.json");
    let char: Vec<Char> = read_json(&char_file)?;
    let mut no_skin_info = char
        .into_iter()
        .filter(|s| s.get_by.as_ref().is_some_and(|s| s != "无") && s.skin1name.is_none())
        .collect::<Vec<_>>();
    no_skin_info.sort_unstable_by(|a, b| b.obtain_date.cmp(&a.obtain_date));

    let data = no_skin_info
        .into_iter()
        .map(|item| {
            Chunk::new(
                img_path(&item.Name).unwrap(),
                vec![item.Name],
                vec![
                    format!("上线: {}", item.obtain_date.as_ref().unwrap()),
                    item.obtain_way.unwrap(),
                    format!(
                        "无皮: {}天",
                        (Local::now().date_naive()
                            - NaiveDate::parse_from_str(&item.obtain_date.unwrap(), "%Y.%m.%d")
                                .unwrap())
                        .num_days()
                    ),
                ],
            )
            .unwrap()
        })
        .collect::<Vec<_>>();

    write_json("NoSkin.json", &data)?;
    Ok(())
}

pub fn birth() -> Result<()> {
    let file_path = Path::new(RESOURCE).join("char");
    let file_oprator: Vec<CharFile> = read_json(&file_path.join("CharFile.json"))?;
    let file_car: Vec<CarFile> = read_json(&file_path.join("CarFile.json"))?;

    let mut birth = file_oprator
        .into_iter()
        .filter(|f| f.dateOfBirth.is_some())
        .map(|f| (f.Name, "生日".to_string(), f.dateOfBirth.unwrap()))
        .chain(
            file_car
                .into_iter()
                .map(|file| (file.代号, "出厂日期".to_string(), file.出厂日)),
        )
        .collect::<Vec<_>>();
    birth.sort_unstable_by(|a, b| a.2.cmp(&b.2));

    let data = birth
        .into_iter()
        .map(|s| Chunk::new(img_path(&s.0).unwrap(), vec![s.0], vec![s.1, s.2]).unwrap())
        .collect::<Vec<Chunk>>();

    write_json("Birth.json", &data)?;
    Ok(())
}

pub fn real_name(file: &str, save_name: &str) -> Result<()> {
    let mut file_path = PathBuf::from(RESOURCE);
    file_path.push(file);
    let file_oprator: Vec<RealName> = read_json(&file_path)?;

    let data = file_oprator
        .into_iter()
        .map(|o| {
            Chunk::new(
                img_path(&o.operator).unwrap(),
                vec![o.operator],
                vec![
                    o.real_name.join("\n"),
                    format!("出处: \n{}", o.source.join("\n")),
                ],
            )
            .unwrap()
        })
        .collect::<Vec<_>>();

    write_json(save_name, &data)?;
    Ok(())
}

type MapType = HashMap<String, Vec<String>>;
pub fn merge_map(mut map1: MapType, map2: MapType) -> MapType {
    for (key, mut value) in map2 {
        map1.entry(key)
            .and_modify(|v| v.append(&mut value))
            .or_insert(value);
    }
    map1
}

pub fn crop() -> Result<()> {
    let file_path = Path::new(RESOURCE).join("char");

    let info: Vec<Char> = read_json(&file_path.join("Char.json"))?;
    let map_info: HashMap<_, _> = info
        .into_iter()
        .map(|s| {
            let temp: Vec<_> = [
                ("性别", s.sex),
                ("战斗经验", s.combatExperience),
                ("出生地", s.birthPlace),
                ("生日", s.dateOfBirth),
                ("种族", s.race),
                ("身高", s.height),
                ("感染状态", s.infectionStatus),
                ("物理强度", s.phy),
                ("战场机动", s.flex),
                ("生理耐受", s.tolerance),
                ("战术规划", s.plan),
                ("战斗技巧", s.skill),
                ("源石技艺适应性", s.adapt),
                ("职业", s.profession),
                ("分支职业", s.subProfession),
                ("位置", s.position),
                ("星级", s.rarity.map(|n| n.to_string())),
                ("tag", s.tag),
                ("上线日期", s.obtain_date),
            ]
            .into_iter()
            .filter_map(|(label, value)| value.map(|v| format!("{label}: {v}")))
            .collect();
            let mut v = if temp.is_empty() {
                temp
            } else {
                simple_rng::random_choose_n(temp, 1)
            };
            let v2 = v.split_off(0);
            (s.Name, (v, v2))
        })
        .collect();
    let mut v = Vec::with_capacity(300);
    for entry in fs::read_dir(r"E:\pictures\foot_output")? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let file_name = path.file_name().unwrap().to_string_lossy();
            if let Some((name, _)) = file_name.split_once('_') {
                if let Some((v1, v2)) = map_info.get(name) {
                    v.push((path.to_owned(), v1.clone(), v2.clone()));
                }
            }
        }
    }
    let len = v.len();
    simple_rng::suffix(&mut v, len);
    let v: Vec<_> = v
        .into_iter()
        .enumerate()
        .map(|(i, mut c)| {
            Chunk::new(
                c.0,
                {
                    c.1.push((i + 1).to_string());
                    c.1
                },
                c.2,
            )
            .unwrap()
        })
        .collect();
    write_json("Crop.json", &v)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name() {
        // if let Err(e) = real_name("RealName.json", "RealName.json") {
        //     println!("{e}");
        // }
        // if let Err(e) = real_name("RealNameNotOp.json", "RealNameNotOp.json") {
        //     println!("{e}");
        // }
        // if let Err(e) = no_skin() {
        //     println!("{e}");
        // }
        // if let Err(e) = birth() {
        //     println!("{e}");
        // }
        if let Err(e) = crop() {
            println!("{e}");
        }
    }
}
