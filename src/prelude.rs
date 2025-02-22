#![allow(unused)]
use std::collections::HashMap;

pub const RESOURCE: &str = "../resources";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name() {
        println!("{:?}", std::fs::canonicalize(RESOURCE));
    }
}

fn _lmain() {
    let _file: HashMap<&str, &str> = HashMap::from([
        ("干员立绘以及小人拆件", "data/com.hypergryph.arknights/files/AB/Android/chararts"),
        ("干员皮肤立绘以及小人拆件", "data/com.hypergryph.arknights/files/AB/Android/skinpack"),
        ("动态立绘的拆件", "data/com.hypergryph.arknights/files/AB/Android/arts/dynchars"),
        ("剧情立绘以及角色表情差分", "data/com.hypergryph.arknights/files/AB/Android/avg/characters"),
        ("剧情CG", "data/com.hypergryph.arknights/files/AB/Android/avg/imgs"),
        ("剧情背景图", "data/com.hypergryph.arknights/files/AB/Android/avg/bg"),
        ("剧情小资源及视频", "data/com.hypergryph.arknights/files/AB/Android/avg/items"),
        ("视频", "data/com.hypergryph.arknights/files/AB/Android/raw"),
        ("剧情文本", "data/com.hypergryph.arknights/files/AB/Android/gamedata/story"),
        ("卡池封面", "data/com.hypergryph.arknights/files/AB/Android/ui/gacha"),
        ("关卡地图", "data/com.hypergryph.arknights/files/AB/Android/arts/ui"),
        ("主线章节关卡背景图", "data/com.hypergryph.arknights/files/AB/Android/ui"),
        ("活动关卡背景图", "data/com.hypergryph.arknights/files/AB/Android/activity"),
        ("主界面壁纸", "data/com.hypergryph.arknights/files/AB/Android/arts/ui/homebackground/wrappe"),
        ("活动主界面图以及UI设计", "data/com.hypergryph.arknights/files/AB/Android/activity"),
    ]);
}
