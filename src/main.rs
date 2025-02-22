// #![allow(unused)]
pub mod error;
mod parse_data;
mod prelude;
pub mod swiping_img;

use error::{Kind, Result};
use std::{fs::File, path::Path};
use swiping_img::{BigImg, Chunk};

fn read_json<P, T>(file: P) -> Result<Vec<T>>
where
    P: AsRef<Path>,
    T: serde::Serialize + serde::de::DeserializeOwned,
{
    let file = File::open(file.as_ref()).map_err(|e| err_new_io!(e))?;
    Ok(serde_json::from_reader(file).map_err(|e| err_new!(Kind::Other, &e.to_string()))?)
}

fn main() -> Result<()> {
    let t = std::time::Instant::now();
    let data_file = Path::new("./data").join("Crop.json");

    let work_dir = Path::new("E:/pictures/arknights/0crop");
    std::fs::create_dir_all(work_dir).map_err(|e| err_new_io!(e))?;

    let data_use = &read_json(data_file)?[..100];

    let si = BigImg::builder(work_dir, &data_use)
        .step(60)
        .video_swip_speed(4)
        .build()?;
    // dbg!(&si);
    si.combain("result.mp4").unwrap();

    println!("cost {} ms", t.elapsed().as_millis());
    Ok(())
}

#[cfg(test)]
mod tests {

    #[test]
    fn test() {}
}
